# Contributing to MacMate / 贡献指南

[English](#english) | [中文](#中文)

---

## English

Thanks for your interest in improving MacMate! This guide covers how to report issues, set up a dev environment, and submit changes.

### Reporting Bugs & Feature Requests

- Search [existing issues](https://github.com/gh-pixel-95813/MacMate/issues) first to avoid duplicates.
- Open a new issue and use the appropriate template (bug / feature).
- For bugs, include: macOS version, Mac (Intel/Apple Silicon), MacMate version, reproduction steps, and logs.

### Development Environment

Requirements:

- macOS 10.15 or later
- Xcode Command Line Tools (`xcode-select --install`)
- Node.js 20+
- pnpm (`npm install -g pnpm`)
- Rust toolchain (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)

Setup:

```bash
git clone https://github.com/gh-pixel-95813/MacMate.git
cd MacMate
pnpm install
pnpm tauri:dev
```

### Branching Strategy

We use a simplified git-flow:

- Fork the repo and create a feature branch from `main` (e.g. `feature/large-files-threshold`).
- Keep branches focused and short-lived.
- Open a Pull Request targeting `main`.

### Code Style

Frontend (TypeScript / Vue):

```bash
pnpm lint         # eslint --fix + prettier --write
pnpm lint:check   # CI uses this; must report 0 errors
```

Backend (Rust):

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

### Commit Conventions

Follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` new feature
- `fix:` bug fix
- `docs:` documentation only
- `chore:` tooling / maintenance

Example: `feat(uninstaller): scan ~/Library application support`

### Testing

All tests must pass before a PR can be merged:

```bash
pnpm test:unit                        # Vitest unit tests
cargo test --manifest-path src-tauri/Cargo.toml
```

### Translations

Localization lives in `src/i18n/*.json` (`zh-CN.json`, `en-US.json`). Both files must keep the same key structure. Add the new key to every locale and keep placeholders (`{count}`, `{size}`) consistent.

---

## 中文

感谢你参与 MacMate 的改进!本指南说明如何反馈问题、搭建开发环境并提交改动。

### 反馈 Bug 与功能建议

- 先在 [已有 issue](https://github.com/gh-pixel-95813/MacMate/issues) 中搜索,避免重复。
- 使用对应模板(bug / feature)新建 issue。
- 提交 bug 时请附:macOS 版本、芯片(Intel / Apple Silicon)、MacMate 版本、复现步骤与日志。

### 开发环境

依赖:

- macOS 10.15 及以上
- Xcode 命令行工具(`xcode-select --install`)
- Node.js 20+
- pnpm(`npm install -g pnpm`)
- Rust 工具链(`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)

初始化:

```bash
git clone https://github.com/gh-pixel-95813/MacMate.git
cd MacMate
pnpm install
pnpm tauri:dev
```

### 分支策略

采用简化的 git-flow:

- Fork 仓库后从 `main` 切出 feature 分支(如 `feature/large-files-threshold`)。
- 分支保持聚焦、生命周期尽量短。
- 向 `main` 提交 Pull Request。

### 代码风格

前端(TypeScript / Vue):

```bash
pnpm lint         # eslint --fix + prettier --write
pnpm lint:check   # CI 使用,必须 0 错误
```

后端(Rust):

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

### 提交规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` 新功能
- `fix:` bug 修复
- `docs:` 仅文档
- `chore:` 工具 / 维护

示例:`feat(uninstaller): 扫描 ~/Library application support`

### 测试要求

合并前所有测试必须通过:

```bash
pnpm test:unit                        # Vitest 单元测试
cargo test --manifest-path src-tauri/Cargo.toml
```

### 翻译贡献

国际化文件位于 `src/i18n/*.json`(`zh-CN.json`、`en-US.json`),两份文件须保持相同的键结构。新增键时请在所有语言中补齐,并保持占位符(`{count}`、`{size}`)一致。
