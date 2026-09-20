# MacMate

<div align="center">

**免费、开源的 macOS 清理工具 — CleanMyMac 的轻量替代品**

[![Release](https://img.shields.io/github/v/release/gh-pixel-95813/MacMate?style=flat-square&label=Release&color=blue)](https://github.com/gh-pixel-95813/MacMate/releases)
[![License](https://img.shields.io/github/license/gh-pixel-95813/MacMate?style=flat-square&label=License&color=green)](LICENSE)
[![Downloads](https://img.shields.io/github/downloads/gh-pixel-95813/MacMate/total?style=flat-square&label=Downloads&color=success)](https://github.com/gh-pixel-95813/MacMate/releases)
[![Homebrew](https://img.shields.io/badge/Homebrew-macmate-orange?style=flat-square)](https://github.com/gh-pixel-95813/homebrew-tap)

---

[![CI](https://github.com/gh-pixel-95813/MacMate/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/gh-pixel-95813/MacMate/actions/workflows/ci.yml)
[![Release](https://github.com/gh-pixel-95813/MacMate/actions/workflows/release.yml/badge.svg)](https://github.com/gh-pixel-95813/MacMate/actions/workflows/release.yml)
[![E2E Tests](https://github.com/gh-pixel-95813/MacMate/actions/workflows/e2e.yml/badge.svg)](https://github.com/gh-pixel-95813/MacMate/actions/workflows/e2e.yml)
[![CodeQL](https://github.com/gh-pixel-95813/MacMate/actions/workflows/codeql.yml/badge.svg?branch=main)](https://github.com/gh-pixel-95813/MacMate/actions/workflows/codeql.yml)
[![Homebrew Tap](https://github.com/gh-pixel-95813/MacMate/actions/workflows/update-homebrew-tap.yml/badge.svg?branch=main)](https://github.com/gh-pixel-95813/MacMate/actions/workflows/update-homebrew-tap.yml)

[![macOS](https://img.shields.io/badge/platform-macOS-lightgrey?style=flat-square&logo=apple&logoColor=white)](https://github.com/gh-pixel-95813/MacMate/releases)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Vue 3](https://img.shields.io/badge/Vue-3-42b883?style=flat-square&logo=vue.js&logoColor=white)](https://vuejs.org/)
[![Tauri 2](https://img.shields.io/badge/Tauri-2-FFC131?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app/)

</div>

---

[English](./README.md) | [简体中文](./README.zh-CN.md)

---

## MacMate 是什么

免费、开源的 macOS 清理工具，基于 Tauri + Vue 3 构建，CleanMyMac 的轻量替代品。

## 功能

- **系统垃圾**：缓存、日志、临时文件、废纸篓
- **应用卸载器**：扫描 `/Applications` 及关联 Library 文件
- **大文件扫描**：阈值扫描 + SHA-256 重复检测
- **隐私清理**：Safari/Chrome/Firefox 历史、Cookie、最近文档

## Smart Selection 安全机制

- `safe`（安全，自动勾选）/ `caution`（谨慎，手动）/ `danger`（危险，仅确认）
- 所有删除走废纸篓，30 天内可恢复

## 安装

### Apple Silicon (M1/M2/M3+)

```bash
brew tap gh-pixel-95813/tap
brew install --cask macmate
```

### Intel

```bash
brew tap gh-pixel-95813/tap
brew install --cask macmate-x64
```

> 或使用一行命令：`brew install --cask gh-pixel-95813/tap/macmate`

或从 [Releases](https://github.com/gh-pixel-95813/MacMate/releases) 下载

> 未签名版本首次运行：系统设置 → 隐私与安全性 → 允许

## 源码构建

```bash
git clone https://github.com/gh-pixel-95813/MacMate.git
cd MacMate
pnpm install
pnpm tauri:dev   # 开发
pnpm tauri:build # 生产构建
```

依赖：macOS 10.15+、Node 20+、pnpm、Rust 工具链

## 技术栈

- **后端**：Rust + Tauri 2
- **前端**：Vue 3 + TypeScript + Tailwind + Pinia + vue-i18n
- **CI/CD**：GitHub Actions
  - [CI](https://github.com/gh-pixel-95813/MacMate/actions/workflows/ci.yml) — lint、单元测试、构建（前端 + Rust）
  - [Release](https://github.com/gh-pixel-95813/MacMate/actions/workflows/release.yml) — universal DMG（Apple Silicon + Intel）、ad-hoc 签名
  - [E2E Tests](https://github.com/gh-pixel-95813/MacMate/actions/workflows/e2e.yml) — 每日端到端测试
  - [CodeQL](https://github.com/gh-pixel-95813/MacMate/actions/workflows/codeql.yml) — 每周安全扫描（Rust + JS/TS）
  - [Homebrew Tap](https://github.com/gh-pixel-95813/MacMate/actions/workflows/update-homebrew-tap.yml) — 发布时自动更新 Cask formula
- **分发**：Homebrew Cask（ARM + x64）+ GitHub Releases

## Release 信息

预构建的 universal DMG 发布在 [Releases](https://github.com/gh-pixel-95813/MacMate/releases) 页面。

| 产物                    | 架构                  | 格式 |
| ----------------------- | --------------------- | ---- |
| `MacMate-universal.dmg` | Apple Silicon + Intel | DMG  |

> DMG 使用 ad-hoc 签名（`signingIdentity: -`）。首次运行：系统设置 → 隐私与安全性 → 允许。

## 贡献

见 [CONTRIBUTING.md](CONTRIBUTING.md)

## License

MIT © MacMate Contributors
