//! macOS Keychain 凭证存储(封装 `security` CLI 命令)。
//!
//! 用于安全保存用户在 Settings 里输入的 GitHub PAT,避免明文落 config.json。
//!
//! 非 macOS 平台静默成功(空实现),便于跨平台开发/CI 构建无错编译。
//! 实际 macOS 上通过 `/usr/bin/security add-generic-password` / `find-generic-password`
//! / `delete-generic-password` 操作系统钥匙串。
//!
//! 服务名固定为 `com.macmate.app`、账户名固定为 `github-token`,避免参数外泄。
//!
//! 核心逻辑抽出为 `*_at(service, account)` 形式,便于在单元测试中用临时 keychain
//! 文件覆盖,不污染用户真实 Keychain。

use crate::types::AppError;

/// 服务名常量:MacMate 在 Keychain 里的固定标识。
pub const SERVICE_NAME: &str = "com.macmate.app";
/// 账户名常量:GitHub PAT 专用账户名。
pub const ACCOUNT_NAME: &str = "github-token";

/// 在 Keychain 写入(或覆盖)一条 generic password。
///
/// macOS 用 `security add-generic-password -U -s <svc> -a <acc> -w <pass>`,
/// `-U` 表示存在则更新。非 macOS 平台静默成功。
#[cfg(target_os = "macos")]
fn save_at(service: &str, account: &str, secret: &str) -> Result<(), AppError> {
    let status = std::process::Command::new("/usr/bin/security")
        .args([
            "add-generic-password",
            "-U",
            "-s",
            service,
            "-a",
            account,
            "-w",
            secret,
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .map_err(|e| AppError::Internal(format!("security spawn failed: {e}")))?;
    if !status.success() {
        return Err(AppError::Internal(format!(
            "security add-generic-password exit {:?}",
            status.code()
        )));
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn save_at(_service: &str, _account: &str, _secret: &str) -> Result<(), AppError> {
    Ok(())
}

/// 从 Keychain 读取 generic password;未找到返回 `Ok(None)`。
#[cfg(target_os = "macos")]
fn get_at(service: &str, account: &str) -> Result<Option<String>, AppError> {
    let output = std::process::Command::new("/usr/bin/security")
        .args([
            "find-generic-password",
            "-s",
            service,
            "-a",
            account,
            "-w",
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| AppError::Internal(format!("security spawn failed: {e}")))?;
    if output.status.success() {
        let mut s = String::from_utf8_lossy(&output.stdout).into_owned();
        // security 通常带尾部 \n,trim 掉。
        while s.ends_with('\n') || s.ends_with('\r') {
            s.pop();
        }
        if s.is_empty() {
            Ok(None)
        } else {
            Ok(Some(s))
        }
    } else {
        // 未找到条目(secKeychainErr -25300)等非致命情况,视为未设置。
        Ok(None)
    }
}

#[cfg(not(target_os = "macos"))]
fn get_at(_service: &str, _account: &str) -> Result<Option<String>, AppError> {
    Ok(None)
}

/// 删除 Keychain 中的 generic password;不存在视为成功。
#[cfg(target_os = "macos")]
fn delete_at(service: &str, account: &str) -> Result<(), AppError> {
    let _ = std::process::Command::new("/usr/bin/security")
        .args(["delete-generic-password", "-s", service, "-a", account])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn delete_at(_service: &str, _account: &str) -> Result<(), AppError> {
    Ok(())
}

/// 保存 GitHub PAT 到 Keychain(默认服务/账户名)。
pub fn save_github_token(token: &str) -> Result<(), AppError> {
    save_at(SERVICE_NAME, ACCOUNT_NAME, token)
}

/// 读取 GitHub PAT;未设置返回 `Ok(None)`。
pub fn get_github_token() -> Result<Option<String>, AppError> {
    get_at(SERVICE_NAME, ACCOUNT_NAME)
}

/// 删除 GitHub PAT(不存在也视为成功)。
pub fn clear_github_token() -> Result<(), AppError> {
    delete_at(SERVICE_NAME, ACCOUNT_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_stable() {
        // 服务名与账户名固定,避免参数外泄到日志或前端。
        assert_eq!(SERVICE_NAME, "com.macmate.app");
        assert_eq!(ACCOUNT_NAME, "github-token");
    }

    // 在 CI(macOS runner)里真实跑一次往返:
    #[cfg(target_os = "macos")]
    #[test]
    fn round_trip_on_macos() {
        let svc = "com.macmate.test";
        let acc = "test-token-roundtrip";
        // 先清理上次可能残留的条目。
        let _ = delete_at(svc, acc);
        // 写入。
        save_at(svc, acc, "ghp_test_secret_12345").unwrap();
        // 读出。
        let got = get_at(svc, acc).unwrap();
        assert_eq!(got.as_deref(), Some("ghp_test_secret_12345"));
        // 覆盖写入。
        save_at(svc, acc, "ghp_new_67890").unwrap();
        let got2 = get_at(svc, acc).unwrap();
        assert_eq!(got2.as_deref(), Some("ghp_new_67890"));
        // 清理。
        delete_at(svc, acc).unwrap();
        assert_eq!(get_at(svc, acc).unwrap(), None);
    }

    // 非 macOS 平台:get / save / delete 全部静默成功。
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn stubs_return_ok() {
        assert!(save_at("s", "a", "x").is_ok());
        assert_eq!(get_at("s", "a").unwrap(), None);
        assert!(delete_at("s", "a").is_ok());
    }
}
