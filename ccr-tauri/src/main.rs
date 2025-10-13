// 🖥️ CCR Tauri 桌面应用入口
//
// 本小姐创建的优雅桌面应用！(￣▽￣)／

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ccr_tauri::commands;

fn main() {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_configs,
            commands::get_current_config,
            commands::get_config,
            commands::switch_config,
            commands::create_config,
            commands::update_config,
            commands::delete_config,
            commands::import_config,
            commands::export_config,
            commands::validate_all,
            commands::get_history,
            commands::list_backups,
            commands::restore_backup,
            commands::get_system_info,
        ])
        .run(tauri::generate_context!("tauri.conf.json"))
        .expect("启动 Tauri 应用失败！");
}
