// MacMate Tauri 库入口:注册命令、暴露公共工具模块。
//
// Task 3 落地的工具层(types / fsutil / trash / sudo / safety)在此声明为模块,
// 供后续 Task 4-7 的扫描器与清理器调用;同时注册一个示例命令 `get_disk_usage`,
// 供 Task 2 的前端 DiskUsageWidget 调用。

// 工具层模块声明为 `pub`:它们是 Task 4-7(扫描器 / 清理器 / 命令)将调用的
// 公共工具 API,也便于集成测试直接引用,同时避免在未被调用前触发 dead_code。
pub mod fsutil;
pub mod safety;
pub mod sudo;
pub mod trash;
mod types;
// Task 6+:功能域命令模块。invoke_handler 由统一注册入口追加,此处仅声明模块
// 以使命令实现可被编译与单元测试覆盖。
pub mod commands;

pub use types::*;

use sysinfo::Disks;

/// 返回根挂载点的磁盘使用概览,供仪表盘 DiskUsageWidget 调用。
///
/// 优先返回挂载点为 `/` 的磁盘;若不存在(非 POSIX 根),回退到第一个磁盘。
#[tauri::command]
fn get_disk_usage() -> Result<DiskUsage, AppError> {
    let disks = Disks::new_with_refreshed_list();
    let root = std::path::Path::new("/");
    let disk = disks
        .list()
        .iter()
        .find(|d| d.mount_point() == root)
        .or_else(|| disks.list().first())
        .ok_or_else(|| AppError::Internal("no disk found by sysinfo".into()))?;
    Ok(DiskUsage {
        total_bytes: disk.total_space(),
        used_bytes: disk.total_space() - disk.available_space(),
        available_bytes: disk.available_space(),
        mount_point: disk.mount_point().to_string_lossy().into_owned(),
    })
}

/// 启动 Tauri 应用,注册命令处理器。
///
/// Task 3 的 `get_disk_usage` + Task 4-7 的四个清理模块命令统一在此注册。
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // 仪表盘
            get_disk_usage,
            // Task 4:系统垃圾清理
            commands::system_junk::scan_system_junk,
            commands::system_junk::clean_system_junk,
            // Task 5:应用卸载器
            commands::uninstaller::scan_applications,
            commands::uninstaller::find_app_related,
            commands::uninstaller::uninstall_app,
            // Task 6:大文件扫描
            commands::large_files::scan_large_files,
            commands::large_files::scan_duplicates,
            commands::large_files::clean_large_files,
            // Task 7:隐私清理
            commands::privacy::scan_privacy,
            commands::privacy::check_browsers_running,
            commands::privacy::clean_privacy,
            // Task 8.2 + 8.3:深度清理(sudo)
            commands::deep_clean::deep_clean_system,
            commands::deep_clean::check_admin_available,
            // Task 8.4:清理历史持久化
            commands::history::get_clean_history,
            commands::history::add_clean_history,
            commands::history::clear_clean_history,
            // Task 9.3:应用配置持久化
            commands::config::get_config,
            commands::config::save_config,
            commands::config::reveal_config_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
