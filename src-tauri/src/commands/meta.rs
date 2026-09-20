//! 应用元信息命令:动态版本号 + 系统/硬件环境信息。
//!
//! - `get_app_version`:从 `tauri::generate_context!()` 暴露的 `config.version`
//!   读出,避免前端硬编码导致 Settings 显示陈旧版本。
//! - `get_system_info`:聚合 macOS 版本 / CPU 架构 / 磁盘容量等环境信息,
//!   供前端在 Settings 关于区展示,也用于上报日志时一并附在 Issue 正文。

use crate::types::{AppError, SystemInfo};
use sysinfo::Disks;

/// 返回应用版本号(取自 `tauri.conf.json` 的 `version` 字段,与 Cargo.toml 一致)。
#[tauri::command]
pub async fn get_app_version() -> Result<String, AppError> {
    // 编译期从 Cargo.toml 读出,与 tauri.conf.json 的 version 字段保持同步。
    // 避免运行时通过 tauri::generate_context!().config() 取值(需推断 Runtime 类型)。
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

/// 返回系统环境信息(用于日志上报与 Settings 关于区展示)。
///
/// - `os_version`:macOS 版本号字符串(从 `sw_vers -productVersion` 读出)。
/// - `arch`:CPU 架构(`aarch64` / `x86_64`)。
/// - `hostname`:主机名。
/// - `disk_total_bytes` / `disk_available_bytes`:根挂载点容量(便于排查磁盘相关问题)。
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, AppError> {
    Ok(collect_system_info())
}

/// 实际收集系统信息,抽出便于单元测试。也供 logs 命令在构造上报 body 时调用。
pub fn collect_system_info() -> SystemInfo {
    let os_version = read_macos_product_version().unwrap_or_else(|| "unknown".into());
    let arch = std::env::consts::ARCH.to_string();
    let hostname = sysinfo::System::host_name().unwrap_or_else(|| "unknown".into());

    let disks = Disks::new_with_refreshed_list();
    let root = std::path::Path::new("/");
    let disk = disks
        .list()
        .iter()
        .find(|d| d.mount_point() == root)
        .or_else(|| disks.list().first());

    let (disk_total_bytes, disk_available_bytes) = match disk {
        Some(d) => (d.total_space(), d.available_space()),
        None => (0, 0),
    };

    SystemInfo {
        os_version,
        arch,
        hostname,
        disk_total_bytes,
        disk_available_bytes,
    }
}

/// 读取 macOS 产品版本号(运行 `sw_vers -productVersion`)。
///
/// 非 macOS 平台返回 `None`(避免命令失败);macOS 上 sw_vers 不存在或失败也返回 None。
#[cfg(target_os = "macos")]
fn read_macos_product_version() -> Option<String> {
    let output = std::process::Command::new("/usr/bin/sw_vers")
        .arg("-productVersion")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(not(target_os = "macos"))]
fn read_macos_product_version() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_system_info_populates_arch() {
        let info = collect_system_info();
        assert!(matches!(info.arch.as_str(), "aarch64" | "x86_64" | "x86" | "arm"));
        assert!(!info.hostname.is_empty());
    }

    #[test]
    fn macos_version_returns_some_on_macos() {
        let v = read_macos_product_version();
        #[cfg(target_os = "macos")]
        {
            assert!(v.is_some());
            let s = v.unwrap();
            assert!(s.contains('.') || s.chars().all(|c| c.is_ascii_digit()));
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert!(v.is_none());
        }
    }
}
