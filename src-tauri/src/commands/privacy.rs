//! 隐私清理命令(Task 7)。
//!
//! - `scan_privacy`:扫描 Safari / Chrome / Firefox 的隐私数据与系统最近文档,
//!   返回可清理项清单(`PrivacyTarget`)。
//! - `check_browsers_running`:检测各浏览器是否正在运行(`BrowserStatus`)。
//! - `clean_privacy`:接收 path 列表,先检测所属浏览器是否运行,运行中则标记失败,
//!   否则移至废纸篓,返回 `CleanOutcome`。
//!
//! 核心扫描逻辑放在平台无关的 `scan_privacy_impl(home, is_running)` 中,
//! `#[tauri::command]` 异步命令仅做包装与平台门控,便于单元测试在 CI 上覆盖。

use crate::fsutil;
use crate::trash;
use crate::types::{AppError, BrowserStatus, CleanOutcome, CleanedItem, FailedItem, PrivacyTarget};
use std::path::{Path, PathBuf};

// SafetyLevel 仅在扫描逻辑(macOS / test)中使用,需同步门控以避免 unused_imports。
#[cfg(any(target_os = "macos", test))]
use crate::types::SafetyLevel;

/// 检测指定浏览器是否正在运行(内部可测试函数)。
///
/// 通过 sysinfo 枚举进程名匹配:Safari → "Safari"、Chrome → "Google Chrome"、Firefox → "firefox"。
/// 未知浏览器名返回 false。在 CI / 非 macOS 环境中进程名不会匹配,自然返回 false。
fn is_browser_running_internal(browser: &str) -> bool {
    let system = sysinfo::System::new_all();
    let target = match browser {
        "Safari" => "Safari",
        "Chrome" => "Google Chrome",
        "Firefox" => "firefox",
        _ => return false,
    };
    system.processes().values().any(|p| p.name() == target)
}

/// 返回路径占用字节数:目录递归累加、文件取 metadata、不存在返回 0。
fn path_size(path: &Path) -> u64 {
    if path.is_dir() {
        fsutil::dir_size(path)
    } else if path.is_file() {
        std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    }
}

/// 根据路径推断所属浏览器(用于 `clean_privacy` 判断运行状态)。
///
/// 优先匹配 `sharedfilelist`(系统最近文档),再依次匹配 Safari / Chrome / Firefox,
/// 其余归为 `System`。
fn browser_for_path(path: &str) -> &str {
    if path.contains("sharedfilelist") {
        "System"
    } else if path.contains("Safari") {
        "Safari"
    } else if path.contains("Chrome") {
        "Chrome"
    } else if path.contains("Firefox") {
        "Firefox"
    } else {
        "System"
    }
}

/// 构造单个 [`PrivacyTarget`]:路径存在时返回 Some,否则 None。
#[cfg(any(target_os = "macos", test))]
fn make_target(
    id: &str,
    browser: &str,
    label: &str,
    path: &Path,
    is_installed: bool,
    is_running: bool,
    safety: SafetyLevel,
) -> Option<PrivacyTarget> {
    if !path.exists() {
        return None;
    }
    Some(PrivacyTarget {
        id: id.to_string(),
        browser: browser.to_string(),
        label: label.to_string(),
        path: path.to_string_lossy().into_owned(),
        size_bytes: path_size(path),
        is_installed,
        is_running,
        safety,
    })
}

/// 扫描给定 home 路径下的浏览器隐私数据与系统最近文档(路径风格为 macOS Library 形态)。
///
/// 平台无关:接受 home 与 `is_running` 闭包,可被测试用任意 tempdir 调用,
/// 也可被 macOS 上的 `scan_privacy` 调用。非 macOS 上不会被命令入口调用(由 cfg 门控)。
#[cfg(any(target_os = "macos", test))]
fn scan_privacy_impl(home: &Path, is_running: impl Fn(&str) -> bool) -> Vec<PrivacyTarget> {
    let mut targets = Vec::new();
    let library = home.join("Library");

    // --- Safari ---
    let safari_data = library.join("Safari");
    let safari_installed = Path::new("/Applications/Safari.app").exists()
        || Path::new("/System/Applications/Safari.app").exists();
    let safari_running = is_running("Safari");
    let safari_items: [(&str, &str, PathBuf, SafetyLevel); 5] = [
        (
            "history",
            "History",
            safari_data.join("History.db"),
            SafetyLevel::Caution,
        ),
        (
            "cookies",
            "Cookies",
            safari_data.join("Cookies.binarycookies"),
            SafetyLevel::Caution,
        ),
        (
            "cache",
            "Cache",
            library.join("Caches/com.apple.Safari"),
            SafetyLevel::Safe,
        ),
        (
            "downloads",
            "Downloads History",
            safari_data.join("Downloads.plist"),
            SafetyLevel::Caution,
        ),
        (
            "form-data",
            "Form Data",
            safari_data.join("Form Values"),
            SafetyLevel::Caution,
        ),
    ];
    for (id_suffix, label, path, safety) in &safari_items {
        let id = format!("safari-{id_suffix}");
        if let Some(t) = make_target(
            &id,
            "Safari",
            label,
            path,
            safari_installed,
            safari_running,
            *safety,
        ) {
            targets.push(t);
        }
    }

    // --- Chrome (Default profile) ---
    let chrome_data = library.join("Application Support/Google/Chrome/Default");
    let chrome_installed = Path::new("/Applications/Google Chrome.app").exists();
    let chrome_running = is_running("Chrome");
    let chrome_items: [(&str, &str, PathBuf, SafetyLevel); 3] = [
        (
            "history",
            "History",
            chrome_data.join("History"),
            SafetyLevel::Caution,
        ),
        (
            "cookies",
            "Cookies",
            chrome_data.join("Cookies"),
            SafetyLevel::Caution,
        ),
        (
            "cache",
            "Cache",
            chrome_data.join("Cache"),
            SafetyLevel::Safe,
        ),
    ];
    for (id_suffix, label, path, safety) in &chrome_items {
        let id = format!("chrome-default-{id_suffix}");
        if let Some(t) = make_target(
            &id,
            "Chrome",
            label,
            path,
            chrome_installed,
            chrome_running,
            *safety,
        ) {
            targets.push(t);
        }
    }
    // Chrome 全局缓存
    let chrome_cache = library.join("Caches/Google/Chrome");
    if let Some(t) = make_target(
        "chrome-global-cache",
        "Chrome",
        "Cache",
        &chrome_cache,
        chrome_installed,
        chrome_running,
        SafetyLevel::Safe,
    ) {
        targets.push(t);
    }

    // --- Firefox (多 Profile) ---
    let firefox_root = library.join("Application Support/Firefox");
    let firefox_installed = Path::new("/Applications/Firefox.app").exists();
    let firefox_running = is_running("Firefox");
    let profiles_dir = firefox_root.join("Profiles");
    if profiles_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&profiles_dir) {
            for entry in entries.flatten() {
                let profile_path = entry.path();
                if !profile_path.is_dir() {
                    continue;
                }
                let profile_name = profile_path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let ff_items: [(&str, &str, PathBuf, SafetyLevel); 3] = [
                    (
                        "history",
                        "History",
                        profile_path.join("places.sqlite"),
                        SafetyLevel::Caution,
                    ),
                    (
                        "cookies",
                        "Cookies",
                        profile_path.join("cookies.sqlite"),
                        SafetyLevel::Caution,
                    ),
                    (
                        "cache",
                        "Cache",
                        profile_path.join("cache2"),
                        SafetyLevel::Safe,
                    ),
                ];
                for (id_suffix, label, path, safety) in &ff_items {
                    let id = format!("firefox-{profile_name}-{id_suffix}");
                    if let Some(t) = make_target(
                        &id,
                        "Firefox",
                        label,
                        path,
                        firefox_installed,
                        firefox_running,
                        *safety,
                    ) {
                        targets.push(t);
                    }
                }
            }
        }
    }

    // --- System: 最近文档 ---
    let recent_dir = library
        .join("Application Support/com.apple.sharedfilelist/com.apple.LSSharedFileList.ApplicationRecentDocuments");
    if recent_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&recent_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let app_id = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let id = format!("system-recent-{app_id}");
                if let Some(t) = make_target(
                    &id,
                    "System",
                    "Recent Documents",
                    &path,
                    true,
                    false,
                    SafetyLevel::Caution,
                ) {
                    targets.push(t);
                }
            }
        }
    }

    targets
}

/// 清理选中项的内部实现(平台无关,可测试)。
///
/// 对每个 path:推断所属浏览器,若该浏览器正在运行则计入 `failed`
/// (reason = "Browser is running, please close it first"),否则移至废纸篓。
fn clean_privacy_impl(items: &[String], is_running: impl Fn(&str) -> bool) -> CleanOutcome {
    let mut success = Vec::new();
    let mut failed = Vec::new();

    for path_str in items {
        let path = PathBuf::from(path_str);
        let browser = browser_for_path(path_str);

        // System 类(最近文档)无需检查浏览器运行状态
        if browser != "System" && is_running(browser) {
            failed.push(FailedItem {
                id: path_str.clone(),
                path: path_str.clone(),
                reason: "Browser is running, please close it first".to_string(),
            });
            continue;
        }

        // 在删除前记录 size,删除后无法再 stat
        let size_bytes = path_size(&path);
        match trash::move_to_trash(std::slice::from_ref(&path)) {
            Ok(()) => success.push(CleanedItem {
                id: path_str.clone(),
                path: path_str.clone(),
                size_bytes,
            }),
            Err(e) => failed.push(FailedItem {
                id: path_str.clone(),
                path: path_str.clone(),
                reason: e.to_string(),
            }),
        }
    }

    CleanOutcome { success, failed }
}

/// 扫描浏览器隐私数据,返回可清理项清单。
///
/// macOS 专属:非 macOS 平台返回空 Vec。
#[tauri::command]
pub async fn scan_privacy() -> Result<Vec<PrivacyTarget>, AppError> {
    #[cfg(target_os = "macos")]
    {
        match dirs::home_dir() {
            Some(home) => Ok(scan_privacy_impl(&home, is_browser_running_internal)),
            None => Ok(Vec::new()),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(Vec::new())
    }
}

/// 检测各浏览器是否正在运行。
#[tauri::command]
pub async fn check_browsers_running() -> Result<Vec<BrowserStatus>, AppError> {
    Ok(vec![
        BrowserStatus {
            browser: "Safari".into(),
            is_running: is_browser_running_internal("Safari"),
        },
        BrowserStatus {
            browser: "Chrome".into(),
            is_running: is_browser_running_internal("Chrome"),
        },
        BrowserStatus {
            browser: "Firefox".into(),
            is_running: is_browser_running_internal("Firefox"),
        },
    ])
}

/// 清理选中的隐私数据项。
///
/// 接收 path 列表,先检测所属浏览器是否运行,运行中则标记失败,否则移至废纸篓。
#[tauri::command]
pub async fn clean_privacy(items: Vec<String>) -> Result<CleanOutcome, AppError> {
    Ok(clean_privacy_impl(&items, is_browser_running_internal))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn scan_lists_safari_items() {
        let home = tempdir().unwrap();
        let safari_dir = home.path().join("Library/Safari");
        fs::create_dir_all(&safari_dir).unwrap();
        fs::write(safari_dir.join("History.db"), "fake history data").unwrap();
        fs::write(safari_dir.join("Cookies.binarycookies"), "fake cookies").unwrap();

        let targets = scan_privacy_impl(home.path(), |_| false);
        let history = targets.iter().find(|t| t.id == "safari-history");
        assert!(history.is_some(), "应列出 Safari 历史记录");
        assert!(history.unwrap().size_bytes > 0);

        let cookies = targets.iter().find(|t| t.id == "safari-cookies");
        assert!(cookies.is_some(), "应列出 Safari Cookies");
    }

    #[test]
    fn scan_lists_chrome_default_profile() {
        let home = tempdir().unwrap();
        let chrome_dir = home
            .path()
            .join("Library/Application Support/Google/Chrome/Default");
        fs::create_dir_all(&chrome_dir).unwrap();
        fs::write(chrome_dir.join("History"), "chrome history").unwrap();
        fs::write(chrome_dir.join("Cookies"), "chrome cookies").unwrap();

        let targets = scan_privacy_impl(home.path(), |_| false);
        assert!(targets.iter().any(|t| t.id == "chrome-default-history"));
        assert!(targets.iter().any(|t| t.id == "chrome-default-cookies"));
    }

    #[test]
    fn scan_lists_firefox_profiles() {
        let home = tempdir().unwrap();
        let profile = home
            .path()
            .join("Library/Application Support/Firefox/Profiles/abc123.default");
        fs::create_dir_all(&profile).unwrap();
        fs::write(profile.join("places.sqlite"), "ff history").unwrap();
        fs::write(profile.join("cookies.sqlite"), "ff cookies").unwrap();
        fs::create_dir_all(profile.join("cache2")).unwrap();

        let targets = scan_privacy_impl(home.path(), |_| false);
        assert!(targets
            .iter()
            .any(|t| t.id == "firefox-abc123.default-history"));
        assert!(targets
            .iter()
            .any(|t| t.id == "firefox-abc123.default-cookies"));
        assert!(targets
            .iter()
            .any(|t| t.id == "firefox-abc123.default-cache"));
    }

    #[test]
    fn scan_lists_system_recent_documents() {
        let home = tempdir().unwrap();
        let recent_dir = home
            .path()
            .join("Library/Application Support/com.apple.sharedfilelist/com.apple.LSSharedFileList.ApplicationRecentDocuments");
        fs::create_dir_all(&recent_dir).unwrap();
        fs::write(recent_dir.join("com.apple.TextEdit"), "recent docs").unwrap();

        let targets = scan_privacy_impl(home.path(), |_| false);
        let recent = targets.iter().find(|t| t.browser == "System");
        assert!(recent.is_some(), "应列出系统最近文档");
        assert_eq!(recent.unwrap().label, "Recent Documents");
        assert_eq!(recent.unwrap().safety, SafetyLevel::Caution);
    }

    #[test]
    fn scan_sets_safety_levels() {
        let home = tempdir().unwrap();
        let safari_dir = home.path().join("Library/Safari");
        fs::create_dir_all(&safari_dir).unwrap();
        fs::write(safari_dir.join("History.db"), "x").unwrap();
        let caches_dir = home.path().join("Library/Caches/com.apple.Safari");
        fs::create_dir_all(&caches_dir).unwrap();
        fs::write(caches_dir.join("cache.bin"), "y").unwrap();

        let targets = scan_privacy_impl(home.path(), |_| false);
        let history = targets.iter().find(|t| t.id == "safari-history").unwrap();
        assert_eq!(history.safety, SafetyLevel::Caution);
        let cache = targets.iter().find(|t| t.id == "safari-cache").unwrap();
        assert_eq!(cache.safety, SafetyLevel::Safe);
    }

    #[test]
    fn scan_empty_home_returns_empty() {
        let home = tempdir().unwrap();
        let targets = scan_privacy_impl(home.path(), |_| false);
        assert!(targets.is_empty(), "空 home 应返回空列表");
    }

    #[test]
    fn clean_removes_safari_history() {
        let home = tempdir().unwrap();
        let safari_dir = home.path().join("Library/Safari");
        fs::create_dir_all(&safari_dir).unwrap();
        let history = safari_dir.join("History.db");
        fs::write(&history, "fake history").unwrap();
        assert!(history.exists());

        let path_str = history.to_string_lossy().into_owned();
        let outcome = clean_privacy_impl(&[path_str], |_| false);
        assert_eq!(outcome.success.len(), 1, "应成功清理 1 项");
        assert_eq!(outcome.failed.len(), 0);
        assert!(!history.exists(), "文件应在清理后从原位置消失");
    }

    #[test]
    fn clean_fails_when_browser_running() {
        let home = tempdir().unwrap();
        let safari_dir = home.path().join("Library/Safari");
        fs::create_dir_all(&safari_dir).unwrap();
        let history = safari_dir.join("History.db");
        fs::write(&history, "fake history").unwrap();

        let path_str = history.to_string_lossy().into_owned();
        // 模拟浏览器运行中
        let outcome = clean_privacy_impl(&[path_str], |browser| browser == "Safari");
        assert_eq!(outcome.success.len(), 0, "运行中不应清理");
        assert_eq!(outcome.failed.len(), 1);
        assert!(
            outcome.failed[0].reason.contains("running"),
            "失败原因应包含 running: {}",
            outcome.failed[0].reason
        );
        assert!(history.exists(), "文件应仍在原位");
    }

    #[test]
    fn clean_system_items_skip_running_check() {
        let home = tempdir().unwrap();
        let recent_dir = home
            .path()
            .join("Library/Application Support/com.apple.sharedfilelist/com.apple.LSSharedFileList.ApplicationRecentDocuments");
        fs::create_dir_all(&recent_dir).unwrap();
        let doc = recent_dir.join("com.apple.TextEdit");
        fs::write(&doc, "recent").unwrap();

        let path_str = doc.to_string_lossy().into_owned();
        // 即使所有浏览器"运行中",System 项也应可清理
        let outcome = clean_privacy_impl(&[path_str], |_| true);
        assert_eq!(
            outcome.success.len(),
            1,
            "System 项不应受浏览器运行状态影响"
        );
        assert_eq!(outcome.failed.len(), 0);
        assert!(!doc.exists());
    }

    #[test]
    fn clean_empty_input_is_ok() {
        let outcome = clean_privacy_impl(&[], |_| false);
        assert!(outcome.success.is_empty());
        assert!(outcome.failed.is_empty());
    }

    #[test]
    fn browser_for_path_classifies_correctly() {
        assert_eq!(
            browser_for_path("/Users/x/Library/Safari/History.db"),
            "Safari"
        );
        assert_eq!(
            browser_for_path("/Users/x/Library/Application Support/Google/Chrome/Default/History"),
            "Chrome"
        );
        assert_eq!(
            browser_for_path(
                "/Users/x/Library/Application Support/Firefox/Profiles/a/places.sqlite"
            ),
            "Firefox"
        );
        assert_eq!(
            browser_for_path("/Users/x/Library/Application Support/com.apple.sharedfilelist/com.apple.LSSharedFileList.ApplicationRecentDocuments/com.apple.Safari"),
            "System"
        );
    }

    #[test]
    fn is_browser_running_returns_bool_without_panic() {
        let _: bool = is_browser_running_internal("Safari");
        let _: bool = is_browser_running_internal("Chrome");
        let _: bool = is_browser_running_internal("Firefox");
        assert!(!is_browser_running_internal("Unknown"));
    }
}
