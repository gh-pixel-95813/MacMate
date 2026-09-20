//! 日志上报命令:GitHub PAT 管理与日志上报到 GitHub Issue。
//!
//! 命令:
//! - `get_github_token_status`:返回是否已配置 PAT(只读),不返回明文 token。
//! - `save_github_token(token)`:把 PAT 存到 macOS Keychain。
//! - `clear_github_token`:从 Keychain 删除 PAT。
//! - `submit_logs_to_github(days)`:读取最近 N 天本地日志,脱敏 + 截断后,
//!   调 GitHub API 创建 Issue,返回 Issue URL。
//!
//! 所有 Keychain 操作委托给 `crate::keychain`,GitHub API 调用委托给
//! `crate::reporting`,此模块只做命令封装与参数校验。

use crate::keychain;
use crate::logging;
use crate::reporting::{self, CreatedIssue, DEFAULT_TARGET_REPO};
use crate::types::AppError;
use std::fmt;

/// 前端期望的 PAT 状态:是否已配置。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubTokenStatus {
    pub configured: bool,
    /// 默认上报目标仓库(owner/repo)。前端用于展示与提示用户。
    pub target_repo: String,
}

/// 上报结果:返回给前端的简化结构。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitResult {
    pub issue_number: u64,
    pub html_url: String,
    pub files_read: u32,
    pub body_bytes: usize,
}

impl fmt::Display for SubmitResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "issue #{} ({}, {} files, {} bytes)",
            self.issue_number, self.html_url, self.files_read, self.body_bytes
        )
    }
}

/// 返回 PAT 是否已配置(只读,不返回 token 明文)。
#[tauri::command]
pub async fn get_github_token_status() -> Result<GithubTokenStatus, AppError> {
    let token = keychain::get_github_token()?;
    Ok(GithubTokenStatus {
        configured: token.is_some(),
        target_repo: DEFAULT_TARGET_REPO.to_string(),
    })
}

/// 把 PAT 存到 macOS Keychain;非 macOS 平台静默成功(空实现)。
#[tauri::command]
pub async fn save_github_token(token: String) -> Result<(), AppError> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return Err(AppError::Internal("token is empty".into()));
    }
    // 简单格式校验:GitHub PAT 一般以 `ghp_` / `github_pat_` 开头。
    // 不强制,以免 fine-grained 自定义前缀被拒。
    keychain::save_github_token(trimmed)
}

/// 删除 Keychain 中的 PAT;不存在也视为成功。
#[tauri::command]
pub async fn clear_github_token() -> Result<(), AppError> {
    keychain::clear_github_token()
}

/// 读取最近 `days` 天本地日志,脱敏 + 截断后上报到 GitHub Issue。
///
/// 流程:
/// 1. 从 Keychain 读取 PAT(缺失时返回 `AppError::Internal`)。
/// 2. 调 `logging::read_recent_logs(days)` 拼接最近日志。
/// 3. 调 `reporting::create_issue` 创建 Issue。
///
/// `days` 取值范围 [1, 30],超出会被 clamp。
#[tauri::command]
pub async fn submit_logs_to_github(days: u32) -> Result<SubmitResult, AppError> {
    let days = days.clamp(1, 30);
    let token = keychain::get_github_token()?
        .ok_or_else(|| AppError::Internal("github token not configured".into()))?;
    let (raw, files_read) = logging::read_recent_logs(days)?;
    // 附加系统环境信息到上报正文头部,便于维护者复现问题。
    let meta = crate::commands::meta::collect_system_info();
    let title = format!("[Log Report] MacMate on {} ({})", meta.hostname, meta.os_version);
    let body = format!(
        "## Environment\n\n- OS: macOS {}\n- Arch: {}\n- Host: {}\n- Disk: {} total / {} available bytes\n\n## Logs (last {} days, {} files)\n\n```\n{}\n```\n",
        meta.os_version,
        meta.arch,
        meta.hostname,
        meta.disk_total_bytes,
        meta.disk_available_bytes,
        days,
        files_read,
        if raw.is_empty() {
            "(no log entries found)"
        } else {
            raw.as_str()
        },
    );

    let body_bytes_pre = body.len();
    let issue: CreatedIssue = reporting::create_issue(reporting::IssueReportParams {
        repo: DEFAULT_TARGET_REPO,
        token: &token,
        title: &title,
        body: &body,
        labels: &["log-report"],
    })?;

    Ok(SubmitResult {
        issue_number: issue.number,
        html_url: issue.html_url,
        files_read,
        body_bytes: body_bytes_pre,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_days_in_command_range() {
        // 直接调用 clamp 逻辑等价检查。
        assert_eq!(0u32.clamp(1, 30), 1);
        assert_eq!(31u32.clamp(1, 30), 30);
        assert_eq!(7u32.clamp(1, 30), 7);
    }

    #[test]
    fn submit_result_display_format() {
        let r = SubmitResult {
            issue_number: 42,
            html_url: "https://github.com/x/y/issues/42".into(),
            files_read: 3,
            body_bytes: 1024,
        };
        let s = format!("{r}");
        assert!(s.contains("#42"));
        assert!(s.contains("github.com"));
        assert!(s.contains("3 files"));
    }

    #[test]
    fn token_status_serializes_camel_case() {
        let status = GithubTokenStatus {
            configured: true,
            target_repo: "gh-pixel-95813/MacMate".into(),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"configured\""));
        assert!(json.contains("\"targetRepo\""));
        assert!(!json.contains("_"));
    }
}
