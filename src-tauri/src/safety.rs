//! Smart Selection 安全等级标签。
//!
//! `classify(path, category)` 基于路径模式与分类返回 [`SafetyLevel`]。
//! 实际规则见同目录 `safety.toml`(纯文档,不被代码读取)。

use crate::types::SafetyLevel;
use std::path::Path;

/// 基于路径模式与分类返回安全等级。
///
/// 规则优先级(先匹配先返回):
/// 1. `Danger`:路径包含 `Preferences`、`.plist`、`Saved Application State`。
/// 2. `Safe`:路径包含 `Caches`、`tmp`、`Temp`、`Trash`、`QuickLook`、`DerivedData`。
/// 3. `Caution`:路径包含 `Logs`、`Application Support`、`History`、`Cookies`。
/// 4. 默认 `Caution`(未知路径偏保守)。
///
/// `category` 参数为后续扩展保留(例如卸载场景下 Application Support 升级为 Caution),
/// 当前实现以路径模式为主。
pub fn classify(path: &Path, _category: &str) -> SafetyLevel {
    let path_str = path.to_string_lossy();

    if path_str.contains("Preferences")
        || path_str.contains(".plist")
        || path_str.contains("Saved Application State")
    {
        return SafetyLevel::Danger;
    }

    if path_str.contains("Caches")
        || path_str.contains("tmp")
        || path_str.contains("Temp")
        || path_str.contains("Trash")
        || path_str.contains("QuickLook")
        || path_str.contains("DerivedData")
    {
        return SafetyLevel::Safe;
    }

    if path_str.contains("Logs")
        || path_str.contains("Application Support")
        || path_str.contains("History")
        || path_str.contains("Cookies")
    {
        return SafetyLevel::Caution;
    }

    SafetyLevel::Caution
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn safe_caches() {
        assert_eq!(
            classify(
                &PathBuf::from("/Users/x/Library/Caches/com.app"),
                "system_junk"
            ),
            SafetyLevel::Safe
        );
    }

    #[test]
    fn safe_tmp() {
        assert_eq!(
            classify(&PathBuf::from("/tmp/macmate-foo"), "system_junk"),
            SafetyLevel::Safe
        );
    }

    #[test]
    fn safe_quicklook_and_deriveddata() {
        assert_eq!(
            classify(
                &PathBuf::from("/Users/x/Library/Caches/com.apple.QuickLook"),
                "system_junk"
            ),
            SafetyLevel::Safe
        );
        assert_eq!(
            classify(
                &PathBuf::from("/Users/x/Library/Developer/Xcode/DerivedData/App-abc"),
                "system_junk"
            ),
            SafetyLevel::Safe
        );
    }

    #[test]
    fn caution_logs() {
        assert_eq!(
            classify(
                &PathBuf::from("/Users/x/Library/Logs/app.log"),
                "system_junk"
            ),
            SafetyLevel::Caution
        );
    }

    #[test]
    fn caution_history_and_cookies() {
        assert_eq!(
            classify(
                &PathBuf::from("/Users/x/Library/Safari/History.db"),
                "privacy"
            ),
            SafetyLevel::Caution
        );
        assert_eq!(
            classify(
                &PathBuf::from("/Users/x/Library/Cookies/Cookies.binarycookies"),
                "privacy"
            ),
            SafetyLevel::Caution
        );
    }

    #[test]
    fn danger_preferences_plist() {
        assert_eq!(
            classify(
                &PathBuf::from("/Users/x/Library/Preferences/com.app.plist"),
                "uninstaller"
            ),
            SafetyLevel::Danger
        );
    }

    #[test]
    fn danger_saved_application_state() {
        assert_eq!(
            classify(
                &PathBuf::from("/Users/x/Library/Saved Application State/com.app.savedState"),
                "uninstaller"
            ),
            SafetyLevel::Danger
        );
    }

    #[test]
    fn default_is_caution() {
        assert_eq!(
            classify(&PathBuf::from("/Users/x/Documents/file.txt"), "large_files"),
            SafetyLevel::Caution
        );
    }
}
