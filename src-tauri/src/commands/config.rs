//! 应用配置持久化命令(Task 9.3)。
//!
//! 配置落盘到 `~/.macmate/config.json`,记录用户在前端 Settings 页面中调整的
//! 语言、主题、深度清理开关与大文件扫描阈值。启动时由前端 `get_config` 加载
//! 到 store,变更时由 `save_config` 持久化。
//!
//! - `get_config`:读取 `~/.macmate/config.json`,文件不存在返回默认值。
//! - `save_config`:写入 `~/.macmate/config.json`(覆盖式)。
//! - `reveal_config_dir`:在 Finder 中打开 `~/.macmate/`(macOS 用 `open`,
//!   非 macOS 静默成功并返回空)。
//!
//! 文件读写用 `std::fs`,JSON 用 `serde_json`。`~/.macmate/` 目录如不存在则创建。
//! 核心逻辑放在平台无关的 `*_at(dir)` 函数中,接收 config 文件所在目录,
//! 便于单元测试用 tempfile 构造假目录覆盖,不污染真实 `~/.macmate`。

use crate::types::{AppConfig, AppError};
use std::path::{Path, PathBuf};

/// 返回 config.json 的路径:`<dir>/config.json`。
fn config_file(dir: &Path) -> PathBuf {
    dir.join("config.json")
}

/// 读取 `<dir>/config.json`。文件不存在或解析失败均返回默认值。
fn get_config_at(dir: &Path) -> Result<AppConfig, AppError> {
    let path = config_file(dir);
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = std::fs::read_to_string(&path)?;
    if content.trim().is_empty() {
        return Ok(AppConfig::default());
    }
    let cfg: AppConfig = serde_json::from_str(&content)
        .map_err(|e| AppError::Internal(format!("config.json parse error: {e}")))?;
    Ok(cfg)
}

/// 覆盖式写入 `<dir>/config.json`(目录不存在则创建)。
fn save_config_at(dir: &Path, config: &AppConfig) -> Result<(), AppError> {
    std::fs::create_dir_all(dir)?;
    let path = config_file(dir);
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| AppError::Internal(format!("config.json serialize error: {e}")))?;
    std::fs::write(&path, json)?;
    Ok(())
}

/// 解析 `~/.macmate` 目录。home 缺失时返回 `AppError::Internal`。
fn macmate_dir() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Internal("cannot resolve home directory".into()))?;
    Ok(home.join(".macmate"))
}

/// 读取应用配置。文件不存在时返回默认值。
#[tauri::command]
pub async fn get_config() -> Result<AppConfig, AppError> {
    let dir = macmate_dir()?;
    get_config_at(&dir)
}

/// 覆盖式写入应用配置。
#[tauri::command]
pub async fn save_config(config: AppConfig) -> Result<(), AppError> {
    let dir = macmate_dir()?;
    save_config_at(&dir, &config)
}

/// 在 Finder 中打开 `~/.macmate/`(macOS 专属)。
///
/// macOS 用 `open <dir>`;非 macOS 平台静默成功(无 Finder 概念),
/// 便于前端在跨平台开发期间无错调用。
#[tauri::command]
pub async fn reveal_config_dir() -> Result<(), AppError> {
    let dir = macmate_dir()?;
    std::fs::create_dir_all(&dir)?;
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&dir)
            .spawn()
            .map_err(|e| AppError::Internal(format!("open config dir failed: {e}")))?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = dir;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn get_returns_default_when_file_missing() {
        let dir = tempdir().unwrap();
        let cfg = get_config_at(dir.path()).unwrap();
        assert_eq!(cfg.locale, "zh-CN");
        assert_eq!(cfg.theme, "system");
        assert!(!cfg.deep_clean);
        assert_eq!(cfg.large_file_threshold_mb, 50);
    }

    #[test]
    fn save_and_get_round_trip() {
        let dir = tempdir().unwrap();
        let cfg = AppConfig {
            locale: "en-US".into(),
            theme: "dark".into(),
            deep_clean: true,
            large_file_threshold_mb: 200,
        };
        save_config_at(dir.path(), &cfg).unwrap();

        let back = get_config_at(dir.path()).unwrap();
        assert_eq!(back.locale, "en-US");
        assert_eq!(back.theme, "dark");
        assert!(back.deep_clean);
        assert_eq!(back.large_file_threshold_mb, 200);
    }

    #[test]
    fn save_creates_directory_if_missing() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join(".macmate");
        assert!(!nested.exists());
        save_config_at(&nested, &AppConfig::default()).unwrap();
        assert!(nested.is_dir());
        let cfg = get_config_at(&nested).unwrap();
        assert_eq!(cfg.locale, "zh-CN");
    }

    #[test]
    fn save_overwrites_existing() {
        let dir = tempdir().unwrap();
        save_config_at(
            dir.path(),
            &AppConfig {
                locale: "zh-CN".into(),
                theme: "light".into(),
                deep_clean: false,
                large_file_threshold_mb: 50,
            },
        )
        .unwrap();
        save_config_at(
            dir.path(),
            &AppConfig {
                locale: "en-US".into(),
                theme: "dark".into(),
                deep_clean: true,
                large_file_threshold_mb: 100,
            },
        )
        .unwrap();
        let cfg = get_config_at(dir.path()).unwrap();
        assert_eq!(cfg.locale, "en-US");
        assert_eq!(cfg.theme, "dark");
        assert!(cfg.deep_clean);
        assert_eq!(cfg.large_file_threshold_mb, 100);
    }

    #[test]
    fn get_returns_default_on_empty_file() {
        let dir = tempdir().unwrap();
        std::fs::write(config_file(dir.path()), "   ").unwrap();
        let cfg = get_config_at(dir.path()).unwrap();
        assert_eq!(cfg.locale, "zh-CN");
    }

    #[test]
    fn get_errors_on_invalid_json() {
        let dir = tempdir().unwrap();
        std::fs::write(config_file(dir.path()), "{ not json").unwrap();
        let res = get_config_at(dir.path());
        assert!(res.is_err());
    }

    #[test]
    fn config_file_path_is_under_dir() {
        let dir = Path::new("/tmp/macmate-config-test");
        assert_eq!(config_file(dir), dir.join("config.json"));
    }
}
