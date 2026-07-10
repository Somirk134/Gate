pub mod commands;
pub mod config;
pub mod discovery;
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
        .manage(commands::certificate::AcmeState::default())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_process::init())
    .plugin(tauri_plugin_updater::Builder::new().build())
    .manage(updater::UpdateState::default())
    .setup(|app| {
            let _window = app
                .get_webview_window("main")
                .ok_or_else(|| TauriError::WindowNotFound)?;
            tray::setup_tray(app.handle())?;
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = app_handle.state::<runtime::ClientRuntimeState>();
                let _ = state.startup_diagnostics().await;
                // 客户端异常退出后，启动时复用已保存的服务器和 Tunnel 配置自动恢复。
                let _ = state.recover_after_startup().await;
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
            commands::discovery::discovery_local_services,
            commands::discovery::discovery_probe_local_service,
            commands::discovery::discovery_remote_ports,
            commands::discovery::discovery_check_remote_port,
            commands::discovery::discovery_diagnose_tunnel,
            commands::tunnel::create_tunnel,
            commands::tunnel::start_tunnel,
            commands::tunnel::stop_tunnel,
            commands::tunnel::restart_tunnel,
            commands::tunnel::edit_tunnel,
            commands::tunnel::delete_tunnel,
            commands::certificate::certificate_list,
            commands::certificate::certificate_detail,
            commands::certificate::certificate_export_pem,
            commands::certificate::certificate_stats,
            commands::certificate::certificate_delete,
            commands::certificate::certificate_validate_import,
            commands::certificate::certificate_import,
            commands::certificate::certificate_auto_renewal_status,
            commands::certificate::certificate_domain_associations,
            commands::certificate::certificate_renew_now,
            commands::certificate::certificate_redeploy,
            commands::certificate::certificate_toggle_auto_renewal,
            commands::certificate::certificate_acme_prepare,
            commands::certificate::certificate_acme_verify,
            commands::backup::backup_export,
            commands::backup::backup_preview,
            commands::backup::backup_restore,
            commands::config::get_config,
            commands::config::set_config,
            commands::config::export_config_file,
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
            updater::check_for_updates,
            updater::download_update,
            updater::install_update,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}
