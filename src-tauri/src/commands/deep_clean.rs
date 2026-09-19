//! 深度清理命令(Task 8.2 + 8.3)。
//!
//! - `deep_clean_system`:接收需要 sudo 清理的系统级路径(如 `/Library/Caches/*`、
//!   `/Library/Logs/*`、`/private/var/log/*`),对每个 path 构造 `rm -rf '<path>'`,
//!   通过 [`crate::sudo::run_with_sudo`] 提权执行。成功(含文件本就不存在)计入
//!   `success`,失败(sudo 拒绝 / 权限错误)计入 `failed`。
//! - `check_admin_available`:转发 [`crate::sudo::is_admin_available`]。
//!
//! `shell_escape` 用单引号包裹路径并将内部单引号转义为 `'\''`,防止 shell 注入。
//! 非 macOS 平台 `deep_clean_system` 直接返回 `AppError::Sudo`。

use crate::sudo;
use crate::types::{AppError, CleanOutcome};

// 以下 import 仅在 macOS(实际提权)或 test(单元测试)路径的 deep_clean_impl 中使用,
// 非 macOS 非 test 构建中 deep_clean_impl 不编译,需同步门控以避免 unused_imports 警告。
#[cfg(any(target_os = "macos", test))]
use crate::fsutil;
#[cfg(any(target_os = "macos", test))]
use crate::types::{CleanedItem, FailedItem};
#[cfg(any(target_os = "macos", test))]
use std::path::Path;

/// 用单引号包裹路径以防 shell 注入:内部单引号转义为 `'\''`。
///
/// 例:`/a/b'c` → `'/a/b'\''c'`,在 sh / bash 中展开为 `/a/b'c` 单个参数。
pub fn shell_escape(path: &str) -> String {
    let escaped = path.replace('\'', "'\\''");
    format!("'{}'", escaped)
}

/// 深度清理内部实现(平台无关,可测试)。
///
/// 对每个 path:用 `fsutil::dir_size` 预先计算 size_bytes,再调用 `rm -rf` 提权删除。
/// `run_sudo` 为可注入的提权闭包,便于单元测试替换为 stub 而不真正调 osascript。
/// 成功(即使文件本就不存在)计入 success;失败计入 failed。
///
/// 仅在 macOS(实际提权)或 test(单元测试)路径中调用,用 cfg 门控避免
/// 非 macOS 非 test 构建触发 dead_code。
#[cfg(any(target_os = "macos", test))]
fn deep_clean_impl(
    paths: &[String],
    run_sudo: impl Fn(&str) -> Result<String, AppError>,
) -> CleanOutcome {
    let mut success = Vec::new();
    let mut failed = Vec::new();
    for path_str in paths {
        // 在删除前记录 size,删除后无法再 stat;不存在路径 dir_size 返回 0。
        let size_bytes = fsutil::dir_size(Path::new(path_str));
        let cmd = format!("rm -rf {}", shell_escape(path_str));
        match run_sudo(&cmd) {
            Ok(_) => success.push(CleanedItem {
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

/// 以管理员权限删除系统级路径(`rm -rf`)。
///
/// macOS 专属:非 macOS 平台返回 `AppError::Sudo("deep clean only on macOS")`。
#[tauri::command]
pub async fn deep_clean_system(paths: Vec<String>) -> Result<CleanOutcome, AppError> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = paths;
        Err(AppError::Sudo("deep clean only on macOS".into()))
    }
    #[cfg(target_os = "macos")]
    {
        Ok(deep_clean_impl(&paths, sudo::run_with_sudo))
    }
}

/// 检测当前是否处于 admin 上下文,供前端决定是否提示密码框。
#[tauri::command]
pub async fn check_admin_available() -> Result<bool, AppError> {
    Ok(sudo::is_admin_available())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_escape_wraps_in_single_quotes() {
        assert_eq!(
            shell_escape("/Library/Caches/com.app"),
            "'/Library/Caches/com.app'"
        );
    }

    #[test]
    fn shell_escape_escapes_internal_single_quote() {
        // 内部单引号 -> '\'' (闭合单引号、转义单引号、重开单引号)
        assert_eq!(shell_escape("/a/b'c"), "'/a/b'\\''c'");
    }

    #[test]
    fn shell_escape_handles_multiple_single_quotes() {
        assert_eq!(shell_escape("a'b'c"), "'a'\\''b'\\''c'");
    }

    #[test]
    fn shell_escape_empty_string() {
        assert_eq!(shell_escape(""), "''");
    }

    #[test]
    fn deep_clean_impl_records_success_via_stub() {
        let paths = vec![
            "/Library/Caches/x".to_string(),
            "/Library/Logs/y".to_string(),
        ];
        // stub sudo 总是成功
        let outcome = deep_clean_impl(&paths, |_| Ok("done".into()));
        assert_eq!(outcome.success.len(), 2);
        assert!(outcome.failed.is_empty());
        assert_eq!(outcome.success[0].path, "/Library/Caches/x");
        assert_eq!(outcome.success[1].path, "/Library/Logs/y");
    }

    #[test]
    fn deep_clean_impl_records_failure_via_stub() {
        let paths = vec!["/private/var/log/z".to_string()];
        let outcome = deep_clean_impl(&paths, |_| Err(AppError::Sudo("user cancelled".into())));
        assert!(outcome.success.is_empty());
        assert_eq!(outcome.failed.len(), 1);
        assert!(outcome.failed[0].reason.contains("user cancelled"));
    }

    #[test]
    fn deep_clean_impl_mixed_success_and_failure() {
        let paths = vec!["/a".to_string(), "/b".to_string(), "/c".to_string()];
        let outcome = deep_clean_impl(&paths, |cmd| {
            if cmd.contains("/b") {
                Err(AppError::Sudo("denied".into()))
            } else {
                Ok("ok".into())
            }
        });
        assert_eq!(outcome.success.len(), 2);
        assert_eq!(outcome.failed.len(), 1);
        assert_eq!(outcome.failed[0].path, "/b");
    }

    #[test]
    fn deep_clean_impl_size_is_zero_for_nonexistent_path() {
        // 不存在的路径 dir_size 返回 0;stub sudo 成功后应记录 size_bytes=0
        let paths = vec!["/nonexistent-macmate-deepclean-test/zzz".to_string()];
        let outcome = deep_clean_impl(&paths, |_| Ok("ok".into()));
        assert_eq!(outcome.success.len(), 1);
        assert_eq!(outcome.success[0].size_bytes, 0);
    }

    #[test]
    fn deep_clean_impl_empty_input_is_ok() {
        let outcome = deep_clean_impl(&[], |_| Ok("ok".into()));
        assert!(outcome.success.is_empty());
        assert!(outcome.failed.is_empty());
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn deep_clean_system_errors_off_macos() {
        // 通过 tauri 内置 runtime 驱动 async 命令,无需直接依赖 tokio。
        let res =
            tauri::async_runtime::block_on(deep_clean_system(vec!["/Library/Caches/x".into()]));
        match res {
            Err(AppError::Sudo(msg)) => assert!(msg.contains("macOS"), "msg={msg}"),
            other => panic!("期望 Sudo 错误,得到 {other:?}"),
        }
    }

    #[test]
    fn check_admin_available_returns_bool_without_panic() {
        let _: bool = tauri::async_runtime::block_on(check_admin_available()).expect("应返回 bool");
    }
}
