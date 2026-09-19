// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// MacMate Tauri 入口,先空实现,后续 Task 3 填充命令与扫描器
fn main() {
    macmate_lib::run()
}
