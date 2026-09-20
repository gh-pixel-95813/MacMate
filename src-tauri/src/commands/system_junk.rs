//! 系统垃圾清理命令(Task 4)。
//!
//! - `scan_system_junk`:扫描 macOS 用户级缓存 / 日志 / 临时目录等垃圾项。
//! - `clean_system_junk`:将选中项(以 path 作为 id)移至废纸篓。
//!
//! 核心扫描逻辑放在平台无关的 `scan_user_paths(home)` 中,
//! `#[tauri::command]` 异步命令仅做包装与平台门控,便于单元测试在 CI 上覆盖。

use crate::fsutil;
use crate::trash;
use crate::types::{AppError, CleanOutcome, CleanedItem, FailedItem, ScanItem, ScanResult};
use std::path::PathBuf;

// 以下 import 仅在被 cfg 门控的 helper(扫描路径相关)中使用,
// 非 macOS 非 test 构建中那些 helper 不编译,需同步门控以避免 unused_imports 警告。
#[cfg(any(target_os = "macos", test))]
use crate::safety;
// SafetyLevel 仅在单元测试中断言时显式命名,非 test 构建中不使用,
// 故仅在 test 下导入(避免 macOS 非 test 构建触发 unused_imports)。
#[cfg(test)]
use crate::types::SafetyLevel;
#[cfg(any(target_os = "macos", test))]
use std::path::Path;

// 分类标签常量:仅在 macOS 实际扫描路径或单元测试中引用,
// 用 cfg 门控以避免非 macOS 非 test 构建触发 dead_code。
#[cfg(any(target_os = "macos", test))]
const CATEGORY_USER_CACHE: &str = "User Cache";
#[cfg(any(target_os = "macos", test))]
const CATEGORY_USER_LOG: &str = "User Log";
#[cfg(any(target_os = "macos", test))]
const CATEGORY_APP_CACHE: &str = "App Cache";
#[cfg(any(target_os = "macos", test))]
const CATEGORY_TEMP: &str = "Temp";
#[cfg(any(target_os = "macos", test))]
const CATEGORY_TRASH: &str = "Trash";
#[cfg(any(target_os = "macos", test))]
const CATEGORY_XCODE_DERIVED: &str = "Xcode DerivedData";
#[cfg(any(target_os = "macos", test))]
const CATEGORY_QUICKLOOK: &str = "QuickLook Cache";

/// 构造单个 [`ScanItem`]:id 即 path,大小与 mtime 由 fsutil 取,
/// safety 由 [`safety::classify`] 复核等级。
#[cfg(any(target_os = "macos", test))]
fn build_item(path: &Path, category: &str) -> ScanItem {
    let size_bytes = fsutil::dir_size(path);
    let modified_at = fsutil::modified_unix(path).unwrap_or(0);
    let safety = safety::classify(path, category);
    let path_str = path.to_string_lossy().into_owned();
    ScanItem {
        id: path_str.clone(),
        path: path_str,
        size_bytes,
        modified_at,
        safety,
        category: category.to_string(),
        label: None,
    }
}

/// 将 `dir` 下的所有子目录作为 [`ScanItem`] 加入 items,跳过读取错误。
#[cfg(any(target_os = "macos", test))]
fn push_subdirs(items: &mut Vec<ScanItem>, dir: &Path, category: &str) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if let Ok(ft) = entry.file_type() {
            if ft.is_dir() {
                items.push(build_item(&entry.path(), category));
            }
        }
    }
}

/// 扫描给定 home 路径下的用户级垃圾目录(路径风格为 macOS Library 形态)。
///
/// 平台无关:接受 home 参数,可被测试用任意 tempdir 调用,
/// 也可被 macOS 上的 `scan_system_junk` 调用。非 macOS 上不会被命令入口调用(由 cfg 门控)。
#[cfg(any(target_os = "macos", test))]
fn scan_user_paths(home: &Path) -> Vec<ScanItem> {
    let mut items = Vec::new();
    let lib = home.join("Library");
    let caches = lib.join("Caches");

    // ~/Library/Caches/*(跳过 com.apple.QuickLook,后者作为独立 QuickLook Cache 项加入)
    if let Ok(entries) = std::fs::read_dir(&caches) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    if entry.file_name().to_string_lossy() == "com.apple.QuickLook" {
                        continue;
                    }
                    items.push(build_item(&entry.path(), CATEGORY_USER_CACHE));
                }
            }
        }
    }

    // ~/Library/Logs/*
    push_subdirs(&mut items, &lib.join("Logs"), CATEGORY_USER_LOG);

    // ~/Library/Application Support/<app>/Caches(只取应用内 Caches 子目录)
    let app_support = lib.join("Application Support");
    if let Ok(entries) = std::fs::read_dir(&app_support) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let app_caches = entry.path().join("Caches");
                    if app_caches.is_dir() {
                        items.push(build_item(&app_caches, CATEGORY_APP_CACHE));
                    }
                }
            }
        }
    }

    // $TMPDIR
    if let Some(tmpdir) = std::env::var_os("TMPDIR") {
        let p = PathBuf::from(tmpdir);
        if p.is_dir() {
            items.push(build_item(&p, CATEGORY_TEMP));
        }
    }
    // /tmp
    let tmp = Path::new("/tmp");
    if tmp.is_dir() {
        items.push(build_item(tmp, CATEGORY_TEMP));
    }
    // /private/var/folders/*(无权限则 read_dir 失败,会被跳过)
    push_subdirs(&mut items, Path::new("/private/var/folders"), CATEGORY_TEMP);

    // ~/.Trash
    let trash_dir = home.join(".Trash");
    if trash_dir.is_dir() {
        items.push(build_item(&trash_dir, CATEGORY_TRASH));
    }

    // ~/Library/Developer/Xcode/DerivedData
    let derived = lib.join("Developer/Xcode/DerivedData");
    if derived.is_dir() {
        items.push(build_item(&derived, CATEGORY_XCODE_DERIVED));
    }

    // ~/Library/Caches/com.apple.QuickLook
    let quicklook = caches.join("com.apple.QuickLook");
    if quicklook.is_dir() {
        items.push(build_item(&quicklook, CATEGORY_QUICKLOOK));
    }

    items
}

/// 汇总 [`ScanItem`] 列表为 [`ScanResult`]:累加 size 与扫描数。
fn summarize(items: Vec<ScanItem>) -> ScanResult {
    let total_size_bytes = items.iter().map(|i| i.size_bytes).sum();
    let scanned_paths = items.len() as u64;
    ScanResult {
        items,
        total_size_bytes,
        scanned_paths,
    }
}

/// 将选中项(参数为 path 列表,简化设计:id 即 path)逐个移至废纸篓。
/// 成功计入 `success`,失败计入 `failed`(reason 用 `error.to_string()`)。
fn clean_system_junk_inner(items: Vec<String>) -> CleanOutcome {
    let mut success = Vec::new();
    let mut failed = Vec::new();
    for path_str in items {
        let path = PathBuf::from(&path_str);
        // 在删除前记录 size,删除后无法再 stat
        let size_bytes = fsutil::dir_size(&path);
        match trash::move_to_trash(std::slice::from_ref(&path)) {
            Ok(()) => success.push(CleanedItem {
                id: path_str.clone(),
                path: path_str,
                size_bytes,
            }),
            Err(e) => failed.push(FailedItem {
                id: path_str.clone(),
                path: path_str,
                reason: e.to_string(),
            }),
        }
    }
    CleanOutcome { success, failed }
}

/// 扫描系统垃圾。macOS 上扫描 `dirs::home_dir()` 下的用户级目录;
/// 非 macOS 平台返回空结果以便 CI 跑通。
#[tauri::command]
pub async fn scan_system_junk() -> Result<ScanResult, AppError> {
    crate::logging::log_command_start("scan_system_junk");
    let items: Vec<ScanItem> = {
        #[cfg(target_os = "macos")]
        {
            match dirs::home_dir() {
                Some(home) => scan_user_paths(&home),
                None => Vec::new(),
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            Vec::new()
        }
    };
    let result = summarize(items);
    crate::logging::log_command_ok(
        "scan_system_junk",
        &format!("items={}, bytes={}", result.items.len(), result.total_size_bytes),
    );
    Ok(result)
}

/// 清理选中项。参数为 path 列表(简化设计:id 即 path)。
#[tauri::command]
pub async fn clean_system_junk(items: Vec<String>) -> Result<CleanOutcome, AppError> {
    crate::logging::log_command_start("clean_system_junk");
    crate::logging::log_command_ok(
        "clean_system_junk",
        &format!("input_items={}", items.len()),
    );
    Ok(clean_system_junk_inner(items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;

    // 防止测试间 HOME 环境变量并发修改导致相互污染;若前序测试 panic 致中毒也自动恢复。
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// 在当前工作目录下创建无 "tmp" 子串的临时 home,避免被 safety::classify
    /// 误判为 Safe(影响 Logs 的 Caution 断言)。同时设置 HOME 以贴近生产路径解析。
    fn fake_home() -> tempfile::TempDir {
        // 显式 prefix 避开 tempfile 默认的 ".tmpXXXX" 前缀带来的 "tmp" 子串匹配。
        tempfile::Builder::new()
            .prefix("macmate-sj-test-")
            .tempdir_in(std::env::current_dir().expect("cwd available"))
            .expect("tempdir in cwd")
    }

    #[test]
    fn scan_finds_user_caches_under_fake_home() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let home = fake_home();
        std::env::set_var("HOME", home.path());

        let caches = home.path().join("Library/Caches/com.example.app");
        fs::create_dir_all(&caches).unwrap();
        fs::write(caches.join("file"), "x").unwrap();

        let items = scan_user_paths(home.path());
        let found = items
            .iter()
            .find(|i| i.path.contains("com.example.app"))
            .expect("应能扫描到用户缓存目录");
        assert_eq!(found.category, CATEGORY_USER_CACHE);
        assert_eq!(found.safety, SafetyLevel::Safe);
        assert_eq!(found.id, found.path, "id 应等于 path");
    }

    #[test]
    fn scan_finds_user_logs_under_fake_home() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let home = fake_home();
        std::env::set_var("HOME", home.path());

        let logs = home.path().join("Library/Logs/com.example.app");
        fs::create_dir_all(&logs).unwrap();
        fs::write(logs.join("app.log"), "abc").unwrap();

        let items = scan_user_paths(home.path());
        let found = items
            .iter()
            .find(|i| i.path.contains("Logs/com.example.app"))
            .expect("应能扫描到用户日志目录");
        assert_eq!(found.category, CATEGORY_USER_LOG);
        assert_eq!(found.safety, SafetyLevel::Caution);
    }

    #[test]
    fn scan_finds_app_support_caches_under_fake_home() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let home = fake_home();
        std::env::set_var("HOME", home.path());

        let app_support = home
            .path()
            .join("Library/Application Support/com.example.app");
        let app_caches = app_support.join("Caches");
        fs::create_dir_all(&app_caches).unwrap();
        fs::write(app_caches.join("blob"), "y").unwrap();
        // 同级目录不应作为 App Cache 出现(只取 Caches 子目录)
        fs::create_dir_all(app_support.join("Containers")).unwrap();

        let items = scan_user_paths(home.path());
        let found = items
            .iter()
            .find(|i| i.path == app_caches.to_string_lossy())
            .expect("应能扫描到应用内 Caches 子目录");
        assert_eq!(found.category, CATEGORY_APP_CACHE);
        assert_eq!(found.safety, SafetyLevel::Safe);
        // Application Support/<app> 本身不应作为单独项出现
        assert!(items
            .iter()
            .all(|i| i.path != app_support.to_string_lossy()));
    }

    #[test]
    fn scan_includes_trash_and_quicklook_under_fake_home() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let home = fake_home();
        std::env::set_var("HOME", home.path());

        let trash_dir = home.path().join(".Trash");
        fs::create_dir_all(&trash_dir).unwrap();
        fs::write(trash_dir.join("deleted.txt"), "z").unwrap();

        let quicklook = home.path().join("Library/Caches/com.apple.QuickLook");
        fs::create_dir_all(&quicklook).unwrap();
        fs::write(quicklook.join("thumbs.db"), "zzz").unwrap();

        let items = scan_user_paths(home.path());
        let trash_item = items
            .iter()
            .find(|i| i.path == trash_dir.to_string_lossy())
            .expect("应扫描到 ~/.Trash");
        assert_eq!(trash_item.category, CATEGORY_TRASH);
        assert_eq!(trash_item.safety, SafetyLevel::Safe);

        let ql_item = items
            .iter()
            .find(|i| i.path == quicklook.to_string_lossy())
            .expect("应扫描到 QuickLook 缓存");
        assert_eq!(ql_item.category, CATEGORY_QUICKLOOK);
        assert_eq!(ql_item.safety, SafetyLevel::Safe);
    }

    #[test]
    fn scan_includes_xcode_derived_data_under_fake_home() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let home = fake_home();
        std::env::set_var("HOME", home.path());

        // spec:扫描的是 DerivedData 父目录本身,而非其下子目录
        let derived = home.path().join("Library/Developer/Xcode/DerivedData");
        fs::create_dir_all(derived.join("App-abcd")).unwrap();
        fs::write(derived.join("App-abcd/Index.bin"), "0").unwrap();

        let items = scan_user_paths(home.path());
        let found = items
            .iter()
            .find(|i| i.path == derived.to_string_lossy())
            .expect("应扫描到 Xcode DerivedData 父目录");
        assert_eq!(found.category, CATEGORY_XCODE_DERIVED);
        assert_eq!(found.safety, SafetyLevel::Safe);
    }

    #[test]
    fn scan_does_not_panic_when_no_macos_dirs_exist() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let home = fake_home();
        std::env::set_var("HOME", home.path());
        // home 下没有任何 macOS 风格目录,scan 应不 panic;具体项数取决于沙箱是否有 /tmp
        let items = scan_user_paths(home.path());
        let _ = items.len();
    }

    #[test]
    fn summarize_aggregates_total_size_and_count() {
        let items = vec![
            ScanItem {
                id: "a".into(),
                path: "/a".into(),
                size_bytes: 100,
                modified_at: 0,
                safety: SafetyLevel::Safe,
                category: "User Cache".into(),
                label: None,
            },
            ScanItem {
                id: "b".into(),
                path: "/b".into(),
                size_bytes: 200,
                modified_at: 0,
                safety: SafetyLevel::Caution,
                category: "User Log".into(),
                label: None,
            },
        ];
        let result = summarize(items);
        assert_eq!(result.total_size_bytes, 300);
        assert_eq!(result.scanned_paths, 2);
        assert_eq!(result.items.len(), 2);
    }

    #[test]
    fn build_item_sets_id_equal_to_path_and_classifies_safety() {
        let dir = tempfile::tempdir().unwrap();
        // 路径含 "Caches",classify 应返回 Safe
        let path = dir.path().join("Caches");
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("file"), "abc").unwrap();
        let item = build_item(&path, CATEGORY_USER_CACHE);
        assert_eq!(item.id, item.path);
        assert_eq!(item.category, CATEGORY_USER_CACHE);
        assert_eq!(item.safety, SafetyLevel::Safe);
        assert!(item.modified_at >= 0);
        assert!(item.size_bytes > 0);
    }

    #[test]
    fn clean_removes_selected_paths() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("to_clean.txt");
        fs::write(&file, "hello").unwrap();
        assert!(file.exists());

        let outcome = clean_system_junk_inner(vec![file.to_string_lossy().into_owned()]);
        assert!(
            outcome.failed.is_empty(),
            "trash 在沙箱应可用: {:?}",
            outcome.failed
        );
        assert_eq!(outcome.success.len(), 1);
        assert_eq!(outcome.success[0].path, file.to_string_lossy());
        assert!(!file.exists(), "文件应在清理后从原位置消失");
    }

    #[test]
    fn clean_empty_input_is_ok() {
        let outcome = clean_system_junk_inner(vec![]);
        assert!(outcome.success.is_empty());
        assert!(outcome.failed.is_empty());
    }

    #[test]
    fn clean_records_failed_for_nonexistent_path() {
        let outcome = clean_system_junk_inner(vec!["/nonexistent-macmate-test/zzz".into()]);
        // 不存在的路径,trash::delete 会失败,计入 failed
        assert_eq!(outcome.success.len(), 0);
        assert_eq!(outcome.failed.len(), 1);
        assert!(!outcome.failed[0].reason.is_empty());
    }
}
