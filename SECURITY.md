# Security Policy / 安全策略

[English](#english) | [中文](#中文)

---

## English

### Reporting a Vulnerability

If you discover a security vulnerability, **do not** open a public GitHub issue. Please report it privately:

1. Go to the repo's **Security** tab → **Report a vulnerability** (GitHub Security Advisory).
2. Include a description, reproduction steps, and (if possible) a proof of concept.
3. Do not disclose the issue publicly until a fix is released.

We will acknowledge receipt within 72 hours and aim to publish a fix and advisory within 90 days.

### Cleaner Safety Commitment

MacMate is a cleaner, so safe deletion is our top concern:

- **All deletes go to Trash** via `NSWorkspace.recycleURLs`. Nothing is unlinked (`rm`) directly, so every removed item is restorable for ~30 days.
- **Smart Selection** only auto-selects `safe` items; `caution` requires manual selection and `danger` requires explicit confirmation.
- The app never deletes system-critical paths without explicit user confirmation.

### Privilege Use (sudo)

- Elevated actions use `osascript` to prompt for the administrator password through the native macOS dialog. **MacMate never sees, stores, or transmits the password.**
- sudo is only requested for optional system-level (deep clean) operations; user-level cleaning runs without any privilege.
- No `sudoers` files are written and no credentials are persisted anywhere.

### Supported Versions

| Version | Supported |
| ------- | --------- |
| 0.1.x   | ✅        |

Older versions are not maintained; please update to the latest release.

---

## 中文

### 报告漏洞

如果你发现安全漏洞,**请不要**创建公开的 GitHub issue,请通过私有渠道报告:

1. 进入仓库的 **Security** 标签页 → **Report a vulnerability**(GitHub Security Advisory)。
2. 附上漏洞描述、复现步骤,如有可能提供 PoC。
3. 在修复版本发布前,请勿公开披露。

我们将在 72 小时内确认收到,并争取在 90 天内发布修复与公告。

### 清理工具的安全承诺

MacMate 是清理工具,安全删除是首要关注点:

- **所有删除走废纸篓**,通过 `NSWorkspace.recycleURLs` 实现。绝不直接 `rm`/unlink,因此每项被删内容均可在约 30 天内恢复。
- **Smart Selection** 仅自动勾选 `safe` 项;`caution` 需手动勾选,`danger` 须显式确认。
- 系统关键路径未经用户明确确认绝不删除。

### 权限使用(sudo)

- 提权操作通过 `osascript` 调用原生 macOS 对话框输入管理员密码。**MacMate 不获取、不存储、不传输密码。**
- sudo 仅用于可选的系统级(深度清理)操作;用户级清理无需任何提权。
- 不写入任何 `sudoers` 文件,不在任何位置持久化凭据。

### 支持的版本

| 版本  | 是否支持 |
| ----- | -------- |
| 0.1.x | ✅       |

旧版本不再维护,请升级到最新发布版本。
