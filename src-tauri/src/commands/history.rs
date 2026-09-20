//! 清理历史持久化命令(Task 8.4)。
//!
//! 历史记录落盘到 `~/.macmate/history.json`,最多保留 100 条(超过删除最旧的)。
//!
//! - `get_clean_history`:读取全部历史;文件不存在返回空 Vec。
//! - `add_clean_history`:追加一条,超过 100 条截断最旧的。
//! - `clear_clean_history`:清空 history.json(写入空数组)。
//!
//! 文件读写用 `std::fs`,JSON 用 `serde_json`。`~/.macmate/` 目录如不存在则创建。
//! 核心逻辑放在平台无关的 `*_at(dir)` 函数中,接收 history 文件所在目录,
//! 便于单元测试用 tempfile 构造假目录覆盖,不污染真实 `~/.macmate`。

use crate::types::{AppError, CleanHistoryEntry};
use std::path::{Path, PathBuf};

/// 历史记录最多保留条数;超过删除最旧的。
const MAX_HISTORY: usize = 100;

/// 返回 history.json 的路径:`<dir>/history.json`。
fn history_file(dir: &Path) -> PathBuf {
    dir.join("history.json")
}

/// 读取 `<dir>/history.json`。文件不存在返回空 Vec;解析失败返回 `AppError::Internal`。
fn get_clean_history_at(dir: &Path) -> Result<Vec<CleanHistoryEntry>, AppError> {
    let path = history_file(dir);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    let entries: Vec<CleanHistoryEntry> = serde_json::from_str(&content)
        .map_err(|e| AppError::Internal(format!("history.json parse error: {e}")))?;
    Ok(entries)
}

/// 追加一条记录到 `<dir>/history.json`,超过 `MAX_HISTORY` 删除最旧的。
fn add_clean_history_at(dir: &Path, entry: CleanHistoryEntry) -> Result<(), AppError> {
    std::fs::create_dir_all(dir)?;
    let mut entries = get_clean_history_at(dir)?;
    entries.push(entry);
    // 超过上限:保留最新的 MAX_HISTORY 条(即丢弃最旧的)
    if entries.len() > MAX_HISTORY {
        let start = entries.len() - MAX_HISTORY;
        entries = entries.split_off(start);
    }
    write_history(dir, &entries)
}

/// 清空 `<dir>/history.json`(写入空数组)。文件不存在也视为成功。
fn clear_clean_history_at(dir: &Path) -> Result<(), AppError> {
    write_history(dir, &[])
}

/// 将 entries 序列化为 JSON 写入 `<dir>/history.json`。
fn write_history(dir: &Path, entries: &[CleanHistoryEntry]) -> Result<(), AppError> {
    std::fs::create_dir_all(dir)?;
    let path = history_file(dir);
    let json = serde_json::to_string_pretty(entries)
        .map_err(|e| AppError::Internal(format!("history.json serialize error: {e}")))?;
    std::fs::write(&path, json)?;
    Ok(())
}

/// 解析 `~/.macmate` 目录。home 缺失时返回 `AppError::Internal`。
fn macmate_dir() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Internal("cannot resolve home directory".into()))?;
    Ok(home.join(".macmate"))
}

/// 读取全部清理历史。文件不存在返回空 Vec。
#[tauri::command]
pub async fn get_clean_history() -> Result<Vec<CleanHistoryEntry>, AppError> {
    let dir = macmate_dir()?;
    get_clean_history_at(&dir)
}

/// 追加一条清理历史。超过 100 条自动删除最旧的。
#[tauri::command]
pub async fn add_clean_history(entry: CleanHistoryEntry) -> Result<(), AppError> {
    crate::logging::log_command_start("add_clean_history");
    crate::logging::log_command_ok(
        "add_clean_history",
        &format!(
            "module={}, freed_bytes={}, success={}, failed={}",
            entry.module, entry.freed_bytes, entry.success_count, entry.failed_count
        ),
    );
    let dir = macmate_dir()?;
    add_clean_history_at(&dir, entry)
}

/// 清空清理历史。
#[tauri::command]
pub async fn clear_clean_history() -> Result<(), AppError> {
    crate::logging::log_command_start("clear_clean_history");
    let dir = macmate_dir()?;
    let r = clear_clean_history_at(&dir);
    crate::logging::log_command_ok("clear_clean_history", "ok");
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_entry(timestamp: i64, module: &str, freed: u64) -> CleanHistoryEntry {
        CleanHistoryEntry {
            timestamp,
            module: module.into(),
            freed_bytes: freed,
            success_count: 1,
            failed_count: 0,
        }
    }

    #[test]
    fn get_returns_empty_when_file_missing() {
        let dir = tempdir().unwrap();
        let entries = get_clean_history_at(dir.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn add_and_get_round_trip() {
        let dir = tempdir().unwrap();
        add_clean_history_at(dir.path(), sample_entry(1_700_000_000, "system_junk", 1024)).unwrap();
        add_clean_history_at(dir.path(), sample_entry(1_700_000_100, "large_files", 2048)).unwrap();

        let entries = get_clean_history_at(dir.path()).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].module, "system_junk");
        assert_eq!(entries[0].freed_bytes, 1024);
        assert_eq!(entries[1].module, "large_files");
        assert_eq!(entries[1].freed_bytes, 2048);
    }

    #[test]
    fn add_creates_directory_if_missing() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join(".macmate");
        assert!(!nested.exists());
        add_clean_history_at(&nested, sample_entry(1, "privacy", 10)).unwrap();
        assert!(nested.is_dir());
        assert_eq!(get_clean_history_at(&nested).unwrap().len(), 1);
    }

    #[test]
    fn clear_empties_history() {
        let dir = tempdir().unwrap();
        add_clean_history_at(dir.path(), sample_entry(1, "system_junk", 100)).unwrap();
        assert_eq!(get_clean_history_at(dir.path()).unwrap().len(), 1);

        clear_clean_history_at(dir.path()).unwrap();
        assert!(get_clean_history_at(dir.path()).unwrap().is_empty());

        // 清空后再写入仍正常
        add_clean_history_at(dir.path(), sample_entry(2, "uninstaller", 50)).unwrap();
        assert_eq!(get_clean_history_at(dir.path()).unwrap().len(), 1);
    }

    #[test]
    fn add_trims_to_max_history_dropping_oldest() {
        let dir = tempdir().unwrap();
        // 写入 MAX_HISTORY + 10 条
        for i in 0..(MAX_HISTORY + 10) {
            add_clean_history_at(dir.path(), sample_entry(i as i64, "system_junk", i as u64))
                .unwrap();
        }
        let entries = get_clean_history_at(dir.path()).unwrap();
        assert_eq!(entries.len(), MAX_HISTORY);
        // 最旧的若干条应已被丢弃:第一条的 timestamp 应为 10(原 index 10)
        assert_eq!(entries[0].timestamp, 10);
        // 最新一条保留:timestamp = MAX_HISTORY + 9
        assert_eq!(entries.last().unwrap().timestamp, (MAX_HISTORY + 9) as i64);
    }

    #[test]
    fn history_file_path_is_under_dir() {
        let dir = Path::new("/tmp/macmate-history-test");
        assert_eq!(history_file(dir), dir.join("history.json"));
    }
}
