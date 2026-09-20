//! Tauri 命令模块:按功能域组织,各子模块声明为 `pub`,
//! 供 `lib.rs` 的 `invoke_handler` 统一注册。

pub mod config;
pub mod deep_clean;
pub mod history;
pub mod large_files;
pub mod logs;
pub mod meta;
pub mod privacy;
pub mod system_junk;
pub mod uninstaller;
