# MacMate

[![Homebrew](https://img.shields.io/badge/Homebrew-macmate-orange)](https://github.com/gh-pixel-95813/homebrew-tap)

[English](#english) | [中文](#中文)

---

## English

### What is MacMate

Free, open-source macOS cleaner built with Tauri + Vue 3. A lightweight alternative to CleanMyMac.

### Features

- **System Junk**: caches, logs, temp files, trash
- **App Uninstaller**: scan `/Applications` and related Library files
- **Large Files**: threshold scan + SHA-256 duplicate detection
- **Privacy**: Safari/Chrome/Firefox history, cookies, recent docs

### Smart Selection

- `safe` (auto-selected) / `caution` (manual) / `danger` (confirm only)
- All deletes go to Trash (recoverable for 30 days)

### Install

```bash
brew install --cask macmate
```

Or download from [Releases](https://github.com/gh-pixel-95813/MacMate/releases)

> First launch on unsigned build: System Settings → Privacy & Security → allow

### Build from source

```bash
git clone https://github.com/gh-pixel-95813/MacMate.git
cd MacMate
pnpm install
pnpm tauri:dev   # development
pnpm tauri:build # production
```

Requires: macOS 10.15+, Node 20+, pnpm, Rust toolchain

### Tech Stack

- **Backend**: Rust + Tauri 2
- **Frontend**: Vue 3 + TypeScript + Tailwind + Pinia + vue-i18n
- **CI**: GitHub Actions (universal binary, signing, notarization)
- **Dist**: Homebrew Cask + GitHub Releases

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

### License

MIT © MacMate Contributors

---

## 中文

### MacMate 是什么

免费、开源的 macOS 清理工具,基于 Tauri + Vue 3 构建,CleanMyMac 的轻量替代品。

### 功能

- **系统垃圾**:缓存、日志、临时文件、废纸篓
- **应用卸载器**:扫描 `/Applications` 及关联 Library 文件
- **大文件扫描**:阈值扫描 + SHA-256 重复检测
- **隐私清理**:Safari/Chrome/Firefox 历史、Cookie、最近文档

### Smart Selection 安全机制

- `safe`(安全,自动勾选)/ `caution`(谨慎,手动)/ `danger`(危险,仅确认)
- 所有删除走废纸篓,30 天内可恢复

### 安装

```bash
brew install --cask macmate
```

或从 [Releases](https://github.com/gh-pixel-95813/MacMate/releases) 下载

> 未签名版本首次运行:系统设置 → 隐私与安全性 → 允许

### 源码构建

```bash
git clone https://github.com/gh-pixel-95813/MacMate.git
cd MacMate
pnpm install
pnpm tauri:dev   # 开发
pnpm tauri:build # 生产构建
```

依赖:macOS 10.15+、Node 20+、pnpm、Rust 工具链

### 技术栈

- **后端**:Rust + Tauri 2
- **前端**:Vue 3 + TypeScript + Tailwind + Pinia + vue-i18n
- **CI**:GitHub Actions(universal binary、签名、公证)
- **分发**:Homebrew Cask + GitHub Releases

### 贡献

见 [CONTRIBUTING.md](CONTRIBUTING.md)

### License

MIT © MacMate Contributors
