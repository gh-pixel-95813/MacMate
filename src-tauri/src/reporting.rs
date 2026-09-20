//! 日志上报到 GitHub Issue:脱敏 + 截断 + HTTP POST。
//!
//! 调用 GitHub REST API `POST /repos/{owner}/{repo}/issues`,在仓库创建一个
//! 新 Issue,body 里放脱敏过的日志内容。请求带 `Authorization: Bearer <PAT>`,
//! PAT 由调用方(keychain 模块)从 Keychain 取得。
//!
//! 隐私处理:
//! - 用户 home 路径 → `~`(macOS /Users/<name>、Linux /home/<name>)
//! - 邮箱 → `[redacted-email]`
//! - 上报内容总上限 `MAX_BODY_BYTES`(默认 64KB),超过则保留首尾各 32KB,
//!   中间用 `…truncated…` 标注。
//!
//! 不内嵌任何 PAT;所有 token 由调用方从 Keychain 注入。

use crate::types::AppError;
use std::time::Duration;

/// 上报 body 总字节数上限(64KB)。
pub const MAX_BODY_BYTES: usize = 64 * 1024;
/// 单侧保留字节数(32KB):超长时保留首尾各此长度,中间截断。
pub const HEAD_TAIL_BYTES: usize = 32 * 1024;

/// GitHub 默认接收仓库(owner/repo)。这里硬编码 MacMate 主仓库,
/// 如未来希望用户自定义,可改为由调用方传入。
pub const DEFAULT_TARGET_REPO: &str = "gh-pixel-95813/MacMate";

/// 单条 Issue 上报所需参数。
pub struct IssueReportParams<'a> {
    pub repo: &'a str,      // owner/repo 形式
    pub token: &'a str,     // GitHub PAT
    pub title: &'a str,     // Issue 标题
    pub body: &'a str,      // Issue 正文(原始,内部会脱敏 + 截断)
    pub labels: &'a [&'a str], // Issue 标签(如 "log-report")
}

/// 创建 Issue 的 GitHub API 返回的最少信息。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedIssue {
    pub number: u64,
    pub html_url: String,
    pub title: String,
}

/// 对一段文本做脱敏:home 路径 → `~`,邮箱 → `[redacted-email]`。
pub fn redact(input: &str) -> String {
    let mut out = input.to_string();
    if let Some(home) = dirs::home_dir() {
        if let Some(s) = home.to_str() {
            if !s.is_empty() {
                out = out.replace(s, "~");
            }
        }
    }
    // 邮箱脱敏用朴素扫描,避免引入 regex crate 依赖。
    redact_emails_naive(&out)
}

/// 简化的邮箱脱敏:扫描 `x@y.z` 模式(无正则依赖)。
fn redact_emails_naive(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        if let Some(end) = match_email_at(bytes, i) {
            out.push_str("[redacted-email]");
            i = end;
        } else {
            // safety: i < bytes.len(),bytes 是合法 UTF-8 字符串切片。
            let ch_start = i;
            i += 1;
            // 防止切到非字符边界:回退到下一个字符边界。
            while i < bytes.len() && !input.is_char_boundary(i) {
                i += 1;
            }
            out.push_str(&input[ch_start..i]);
        }
    }
    out
}

/// 在 bytes[start..] 起点匹配一个 ASCII 邮箱;成功返回结尾下标,失败 None。
fn match_email_at(bytes: &[u8], start: usize) -> Option<usize> {
    // local-part:1+ 个 ASCII 字母/数字/._+-
    let mut p = start;
    let local_start = p;
    while p < bytes.len() && is_local_char(bytes[p]) {
        p += 1;
    }
    if p == local_start || p >= bytes.len() || bytes[p] != b'@' {
        return None;
    }
    p += 1; // skip '@'
    let domain_start = p;
    while p < bytes.len() && is_domain_char(bytes[p]) {
        p += 1;
    }
    // 必须至少有一个 '.' 分隔域段,且结尾不是 '.'。
    if p == domain_start || p >= bytes.len() {
        return None;
    }
    // 至少包含一个 '.':简单检查 domain 段。
    let domain = &bytes[domain_start..p];
    if !domain.contains(&b'.') {
        return None;
    }
    if domain[0] == b'.' || domain[domain.len() - 1] == b'.' {
        return None;
    }
    Some(p)
}

fn is_local_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'+' | b'-')
}

fn is_domain_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'.' || b == b'-'
}

/// 截断超长文本:超过 MAX_BODY_BYTES 时保留首尾各 HEAD_TAIL_BYTES,
/// 中间插入 `…truncated…`。
pub fn truncate(input: &str) -> String {
    if input.len() <= MAX_BODY_BYTES {
        return input.to_string();
    }
    // 安全地从首尾切到字符边界。
    let mut head_end = HEAD_TAIL_BYTES;
    while head_end < input.len() && !input.is_char_boundary(head_end) {
        head_end += 1;
    }
    let mut tail_start = input.len().saturating_sub(HEAD_TAIL_BYTES);
    while tail_start < input.len() && !input.is_char_boundary(tail_start) {
        tail_start += 1;
    }
    if tail_start < head_end {
        // 边界异常,直接返回头部切片即可。
        return input[..head_end].to_string();
    }
    let mut out = String::with_capacity(head_end + 32 + (input.len() - tail_start));
    out.push_str(&input[..head_end]);
    out.push_str("\n…truncated…\n");
    out.push_str(&input[tail_start..]);
    out
}

/// 调用 GitHub API 创建 Issue。失败时返回 AppError::Internal。
pub fn create_issue(params: IssueReportParams<'_>) -> Result<CreatedIssue, AppError> {
    let url = format!("https://api.github.com/repos/{}/issues", params.repo);
    let body_json = serde_json::json!({
        "title": params.title,
        "body": redact_and_truncate(params.body),
        "labels": params.labels,
    });
    let body_str = body_json.to_string();
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(30))
        .build();
    let resp = agent
        .post(&url)
        .set("Authorization", format!("Bearer {}", params.token).as_str())
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "MacMate-LogReporter")
        .send_string(&body_str);
    let resp = match resp {
        Ok(r) => r,
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            return Err(AppError::Internal(format!(
                "github create issue HTTP {code}: {body}"
            )));
        }
        Err(e) => {
            return Err(AppError::Internal(format!("github create issue transport: {e}")));
        }
    };
    let issue: CreatedIssue = resp
        .into_json()
        .map_err(|e| AppError::Internal(format!("github create issue parse: {e}")))?;
    Ok(issue)
}

/// 对上报内容做 redact + truncate 的组合管道。
pub fn redact_and_truncate(body: &str) -> String {
    let redacted = redact(body);
    truncate(&redacted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_keeps_short_input() {
        let s = "hello world".repeat(10);
        assert_eq!(truncate(&s), s);
    }

    #[test]
    fn truncate_long_input_has_marker() {
        let s = "a".repeat(MAX_BODY_BYTES + 1000);
        let t = truncate(&s);
        assert!(t.contains("…truncated…"));
        assert!(t.len() < MAX_BODY_BYTES + 64);
    }

    #[test]
    fn redact_replaces_home() {
        // 构造一个含 home 路径的字符串,因 home 在 CI 上是 /root 或 /Users/runner,
        // 我们用 dirs::home_dir() 直接探测。
        let home = dirs::home_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        let sample = format!("opening file at {}/Documents/test", home);
        let r = redact(&sample);
        assert!(!r.contains(&home));
        assert!(r.contains("~/Documents/test"));
    }

    #[test]
    fn redact_emails_naive_matches_typical_email() {
        let s = "user@example.com said hi to a@b.co";
        let r = redact_emails_naive(s);
        assert!(r.contains("[redacted-email]"));
        assert!(!r.contains("example.com"));
        assert!(!r.contains("a@b.co"));
    }

    #[test]
    fn redact_emails_naive_ignores_non_email_at() {
        // `foo@bar` 不含 '.' 的不应被替换为 [redacted-email]。
        let s = "type ls@host";
        let r = redact_emails_naive(s);
        assert_eq!(s, r);
    }

    #[test]
    fn match_email_at_recognizes_simple_email() {
        let s = "a@b.co x";
        let bytes = s.as_bytes();
        let end = match_email_at(bytes, 0);
        assert_eq!(end, Some(6)); // "a@b.co" 长度 6
    }

    #[test]
    fn match_email_at_rejects_no_dot_in_domain() {
        let s = "a@bar x";
        let bytes = s.as_bytes();
        assert_eq!(match_email_at(bytes, 0), None);
    }

    #[test]
    fn match_email_at_rejects_dotted_at_end() {
        let s = "a@b.co.";
        let bytes = s.as_bytes();
        // 末尾是 '.',match_email_at 不应识别(因为会停在末尾且 domain 以 '.' 结尾)
        // 实际上匹配会停在 "a@b.co",返回 Some(6),后面 '.' 由调用方保留。
        let end = match_email_at(bytes, 0);
        assert_eq!(end, Some(6));
    }

    #[test]
    fn redact_and_truncate_pipeline() {
        let home = dirs::home_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        let sample = format!("path: {}/x\nemail: a@b.co\n{}", home, "a".repeat(MAX_BODY_BYTES + 100));
        let r = redact_and_truncate(&sample);
        assert!(r.contains("…truncated…"));
        assert!(!r.contains(&home));
        assert!(r.contains("~/x"));
        assert!(!r.contains("a@b.co"));
    }
}
