pub mod commands;
pub mod config;
pub mod native;
pub mod tray;
pub mod updater;
pub mod utils;
pub mod windows;

use anyhow::Result;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gate_client=info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::init())
        .setup(|app| {
            let _window = app.get_webview_window("main").unwrap();
            tray::setup_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::connection::connect,
            commands::connection::disconnect,
            commands::tunnel::create_tunnel,
            commands::tunnel::delete_tunnel,
            commands::config::get_config,
            commands::config::set_config,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
