# MacMate

<div align="center">

**Free, open-source macOS cleaner — a lightweight alternative to CleanMyMac**

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

## What is MacMate

Free, open-source macOS cleaner built with Tauri + Vue 3. A lightweight alternative to CleanMyMac.

## Features

- **System Junk**: caches, logs, temp files, trash
- **App Uninstaller**: scan `/Applications` and related Library files
- **Large Files**: threshold scan + SHA-256 duplicate detection
- **Privacy**: Safari/Chrome/Firefox history, cookies, recent docs

## Smart Selection

- `safe` (auto-selected) / `caution` (manual) / `danger` (confirm only)
- All deletes go to Trash (recoverable for 30 days)

## Install

```bash
brew tap gh-pixel-95813/tap
brew install --cask macmate
```

Or download from [Releases](https://github.com/gh-pixel-95813/MacMate/releases)

> First launch on unsigned build: System Settings → Privacy & Security → allow

## Build from source

```bash
git clone https://github.com/gh-pixel-95813/MacMate.git
cd MacMate
pnpm install
pnpm tauri:dev   # development
pnpm tauri:build # production
```

Requires: macOS 10.15+, Node 20+, pnpm, Rust toolchain

## Tech Stack

- **Backend**: Rust + Tauri 2
- **Frontend**: Vue 3 + TypeScript + Tailwind + Pinia + vue-i18n
- **CI/CD**: GitHub Actions
  - [CI](https://github.com/gh-pixel-95813/MacMate/actions/workflows/ci.yml) — lint, unit tests, build (frontend + Rust)
  - [Release](https://github.com/gh-pixel-95813/MacMate/actions/workflows/release.yml) — universal DMG (Apple Silicon + Intel), ad-hoc signing
  - [E2E Tests](https://github.com/gh-pixel-95813/MacMate/actions/workflows/e2e.yml) — nightly end-to-end testing
  - [CodeQL](https://github.com/gh-pixel-95813/MacMate/actions/workflows/codeql.yml) — weekly security scanning (Rust + JS/TS)
  - [Homebrew Tap](https://github.com/gh-pixel-95813/MacMate/actions/workflows/update-homebrew-tap.yml) — auto-update Cask formula on release
- **Dist**: Homebrew Cask (ARM + x64) + GitHub Releases

## Releases

Pre-built universal DMGs are published on the [Releases](https://github.com/gh-pixel-95813/MacMate/releases) page.

| Artifact                | Architecture          | Format |
| ----------------------- | --------------------- | ------ |
| `MacMate-universal.dmg` | Apple Silicon + Intel | DMG    |

> The DMG is ad-hoc signed (`signingIdentity: -`). On first launch: System Settings → Privacy & Security → allow.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

## License

MIT © MacMate Contributors
