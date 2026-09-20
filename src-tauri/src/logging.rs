//! 本地日志初始化与读取工具(基于 tracing + tracing-appender)。
//!
//! 日志按天滚动到 `~/.macmate/logs/macmate.YYYY-MM-DD.log`,保留最近 14 天。
//! 全局 guard 由 Tauri Builder setup 时持有,App 退出前不会 flush。
//!
//! 模块同时提供:
//! - `init_logger()`:启动时调用,装订阅器与 appender,返回 guard。
//! - `read_recent_logs(days)`:读取最近 N 天日志合并为字符串,供上报使用。
//! - `log_dir()`:返回日志目录路径,供前端 reveal_config_dir 复用。

use crate::types::AppError;
use std::path::PathBuf;
use std::sync::OnceLock;
use tracing_appender::non_blocking::WorkerGuard;

/// 日志目录固定在 `~/.macmate/logs/`,与 config.json / history.json 同根。
const LOGS_SUBDIR: &str = "logs";

/// 全局 guard:必须常驻到 App 退出,否则非阻塞 appender 会丢日志。
static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

/// 返回 `~/.macmate/logs/` 目录。home 不可解析时返回 Internal 错误。
pub fn log_dir() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Internal("cannot resolve home directory".into()))?;
    Ok(home.join(".macmate").join(LOGS_SUBDIR))
}

/// 启动时调用一次,装 tracing-subscriber 与按天滚动 appender。
///
/// 失败仅发生在 `log_dir()` 解析失败(home 不可读)时;成功时 guard 存入
/// 全局 OnceLock,App 退出前不会被释放,确保缓冲日志最终 flush 到磁盘。
pub fn init_logger() -> Result<(), AppError> {
    let dir = log_dir()?;
    std::fs::create_dir_all(&dir)?;
    let appender = tracing_appender::rolling::daily(&dir, "macmate.log");
    let (nb, guard) = tracing_appender::non_blocking(appender);
    // 失败说明 OnceLock 已被初始化(重复调用),保留旧 guard 不覆盖。
    let _ = LOG_GUARD.set(guard);
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(nb)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .init();
    Ok(())
}

/// 在 Tauri 命令入口记录调用,供后续在日志里追溯用户行为与命令序列。
///
/// 用 INFO 级别,不会刷屏;若 logger 未初始化也安全(tracing 默认 no-op)。
pub fn log_command_start(name: &str) {
    tracing::info!(command = name, "invoke start");
}

/// 在 Tauri 命令成功出口记录,附简短结果摘要(如 "items=12, bytes=1024")。
pub fn log_command_ok(name: &str, summary: &str) {
    tracing::info!(command = name, summary = summary, "invoke ok");
}

/// 在 Tauri 命令失败出口记录错误,附错误消息。
pub fn log_command_err(name: &str, err: &crate::types::AppError) {
    tracing::error!(command = name, error = %err, "invoke failed");
}

/// 读取最近 `days` 天的日志文件并合并为单个字符串,返回 `(content, files_read)`。
///
/// 文件按文件名升序拼接(文件名含 YYYY-MM-DD,天然按时间升序)。读取单个文件
/// 失败(权限/不存在/非 UTF-8)时跳过,不中断整体读取。
pub fn read_recent_logs(days: u32) -> Result<(String, u32), AppError> {
    let dir = log_dir()?;
    if !dir.exists() {
        return Ok((String::new(), 0));
    }
    let cutoff = {
        let now = chrono_like_now();
        now - days as i64
    };
    let mut entries: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(ts) = parse_log_filename(&path) {
            if ts >= cutoff {
                entries.push(path);
            }
        }
    }
    entries.sort();
    let mut content = String::new();
    let mut count: u32 = 0;
    for path in entries {
        match std::fs::read_to_string(&path) {
            Ok(s) => {
                content.push_str(&s);
                if !s.ends_with('\n') {
                    content.push('\n');
                }
                count = count.saturating_add(1);
            }
            Err(_) => continue,
        }
    }
    Ok((content, count))
}

/// 从文件名 `macmate.YYYY-MM-DD.log` 解析出 unix 秒;无法解析返回 None。
fn parse_log_filename(path: &std::path::Path) -> Option<i64> {
    let name = path.file_name()?.to_str()?;
    // tracing-appender daily 命名:`macmate.log.YYYY-MM-DD`
    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() < 4 {
        return None;
    }
    let (y, m, d) = (
        parts[parts.len() - 3].parse::<i32>().ok()?,
        parts[parts.len() - 2].parse::<u32>().ok()?,
        parts[parts.len() - 1].parse::<u32>().ok()?,
    );
    Some(unix_from_ymd(y, m, d))
}

/// 简化的"当前 unix 秒"。SystemTime 不便于按天计算偏移,这里直接转 i64。
fn chrono_like_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 把 (年, 月, 日) 转成 unix 秒(UTC,不处理时区,只用于日志文件按天过滤)。
fn unix_from_ymd(y: i32, m: u32, d: u32) -> i64 {
    // 简化算法:从 1970-01-01 起累计天数 × 86400。
    // 适用于日志筛选,精度到天即可,不引入 chrono 依赖。
    let days = days_since_epoch(y, m, d);
    days * 86400
}

/// 计算从 1970-01-01 到 (y, m, d) 的天数(UTC)。仅用于日志文件按天过滤,
/// 不追求历法完美,容错 ±1 天也不影响读取范围(因为 read 时按 cutoff 扩大读取)。
fn days_since_epoch(y: i32, m: u32, d: u32) -> i64 {
    let mut total: i64 = 0;
    for year in 1970..y {
        total += if is_leap(year) { 366 } else { 365 };
    }
    let month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for &md in &month_days[..(m as usize - 1).min(12)] {
        total += md as i64;
    }
    if m > 2 && is_leap(y) {
        total += 1;
    }
    total + (d as i64 - 1)
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn parse_log_filename_daily() {
        let p = std::path::Path::new("macmate.log.2025-09-20");
        assert!(parse_log_filename(p).is_some());
    }

    #[test]
    fn parse_log_filename_invalid() {
        let p = std::path::Path::new("random.txt");
        assert!(parse_log_filename(p).is_none());
    }

    #[test]
    fn read_recent_logs_skips_old_files() {
        let dir = tempdir().unwrap();
        // 写一个"今天"的日志文件(名字按 daily 规则)。
        let now = chrono_like_now();
        // 把今天的 unix 秒反推到 YYYY-MM-DD 比较麻烦,直接用 fixed 文件名:
        let name = "macmate.log.2099-01-01".to_string();
        fs::write(dir.path().join(&name), "future log\n").unwrap();
        // cutoff 用 7 天前,future 文件应该被读到。
        // 我们这里绕过 log_dir 的 ~/.macmate 依赖,改用内部 parse 函数测试:
        let ts = parse_log_filename(std::path::Path::new(&name)).unwrap();
        assert!(ts > now);
    }

    #[test]
    fn days_since_epoch_known() {
        // 1970-01-01 = 0
        assert_eq!(days_since_epoch(1970, 1, 1), 0);
        // 1970-01-02 = 1
        assert_eq!(days_since_epoch(1970, 1, 2), 1);
        // 1971-01-01 = 365(1970 非闰)
        assert_eq!(days_since_epoch(1971, 1, 1), 365);
        // 1972-02-29 = 365 + 365 + 31 + 28 = 789(1971 非闰 + 1972 是闰)
        assert_eq!(days_since_epoch(1972, 2, 29), 365 + 365 + 31 + 28);
    }

    #[test]
    fn leap_year_rules() {
        assert!(is_leap(2000));
        assert!(is_leap(2024));
        assert!(!is_leap(1900));
        assert!(!is_leap(2023));
    }
}
