//! sudo 提权封装。
//!
//! macOS 上通过 `osascript -e 'do shell script "..." with administrator privileges'`
//! 弹出系统密码框授权,不存储密码。非 macOS 平台返回错误 stub。

use crate::types::AppError;

#[cfg(target_os = "macos")]
use std::process::Command;

/// 通过 `osascript` 以管理员权限执行 shell 命令,返回 stdout。
///
/// 实现:`osascript -e 'do shell script "<command>" with administrator privileges'`。
/// macOS 会弹出系统密码框,用户输入后授权本次执行,密码不落盘。
/// 失败时返回 `AppError::Sudo(stderr 或退出码信息)`。
#[cfg(target_os = "macos")]
pub fn run_with_sudo(command: &str) -> Result<String, AppError> {
    // 转义 AppleScript 字串中的反斜杠与双引号。
    let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        r#"do shell script "{}" with administrator privileges"#,
        escaped
    );
    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(AppError::Io)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let code = output.status.code();
        let msg = match code {
            Some(c) => format!("osascript exit {c}: {stderr}"),
            None => format!("osascript terminated: {stderr}"),
        };
        return Err(AppError::Sudo(msg));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// 非 macOS 平台 stub:sudo 仅在 macOS 上支持。
#[cfg(not(target_os = "macos"))]
pub fn run_with_sudo(_command: &str) -> Result<String, AppError> {
    Err(AppError::Sudo("sudo only supported on macOS".into()))
}

/// 检测当前是否处于 admin 上下文(简化判定:`SUDO_USER` 环境变量存在即视为是)。
pub fn is_admin_available() -> bool {
    std::env::var("SUDO_USER").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_admin_available_returns_bool() {
        // 仅验证返回类型与调用不 panic,不真正提权。
        let _: bool = is_admin_available();
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn run_with_sudo_errors_off_macos() {
        let res = run_with_sudo("ls");
        match res {
            Err(AppError::Sudo(msg)) => assert!(msg.contains("macOS")),
            other => panic!("期望 Sudo 错误,得到 {other:?}"),
        }
    }
}
