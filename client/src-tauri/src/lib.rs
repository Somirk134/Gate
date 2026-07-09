pub mod commands;
pub mod config;
pub mod native;
pub mod project;
pub mod runtime;
pub mod tray;
pub mod updater;
pub mod utils;
pub mod windows;

use anyhow::Result;
use tauri::{Error as TauriError, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gate_client=info".into()),
        )
        .init();

    tauri::Builder::default()
        .manage(runtime::ClientRuntimeState::default())
        .manage(project::ProjectWorkspaceState::default())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let _window = app
                .get_webview_window("main")
                .ok_or_else(|| TauriError::WindowNotFound)?;
            tray::setup_tray(app.handle())?;
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = app_handle.state::<runtime::ClientRuntimeState>();
                let _ = state.startup_diagnostics().await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::connection::connect,
            commands::connection::disconnect,
            commands::connection::heartbeat,
            commands::server::server_list,
            commands::server::server_create,
            commands::server::server_update,
            commands::server::server_delete,
            commands::server::server_connect,
            commands::server::server_disconnect,
            commands::server::server_test,
            commands::tunnel::create_tunnel,
            commands::tunnel::start_tunnel,
            commands::tunnel::stop_tunnel,
            commands::tunnel::restart_tunnel,
            commands::tunnel::edit_tunnel,
            commands::tunnel::delete_tunnel,
            commands::certificate::certificate_list,
            commands::certificate::certificate_detail,
            commands::certificate::certificate_export_pem,
            commands::config::get_config,
            commands::config::set_config,
            commands::runtime::runtime_get_statistics,
            commands::runtime::runtime_get_dashboard,
            commands::runtime::runtime_get_health,
            commands::runtime::runtime_collect_metrics,
            commands::runtime::runtime_get_logs,
            commands::runtime::runtime_clear_logs,
            commands::runtime::runtime_get_store_report,
            commands::runtime::runtime_run_startup_diagnostics,
            commands::diagnostics::diagnostics_test_connection,
            commands::diagnostics::diagnostics_run_deployment,
            commands::diagnostics::diagnostics_collect_system_info,
            commands::project::project_list,
            commands::project::project_detail,
            commands::project::project_create,
            commands::project::project_update,
            commands::project::project_delete_impact,
            commands::project::project_delete,
            commands::project::project_set_favorite,
            commands::project::project_set_pinned,
            commands::project::project_add_tunnel,
            commands::project::project_remove_tunnel,
            commands::project::project_move_tunnel,
            commands::project::project_add_domain,
            commands::project::project_remove_domain,
            commands::project::project_add_certificate,
            commands::project::project_remove_certificate,
            commands::project::project_templates,
            commands::project::project_recommend_tunnels,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
