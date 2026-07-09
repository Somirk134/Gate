use tauri::State;

use crate::commands::error::{AppError, CommandResult};
use crate::runtime::{ClientRuntimeState, UpdateTunnelRequest};

#[tauri::command]
pub async fn create_tunnel(
    state: State<'_, ClientRuntimeState>,
    local_port: u16,
    remote_port: u16,
    protocol: String,
    local_host: Option<String>,
    host: Option<String>,
    path: Option<String>,
) -> CommandResult<String> {
    state
        .create_tunnel(local_port, remote_port, protocol, local_host, host, path)
        .await
        .map_err(tunnel_error)
}

#[tauri::command]
pub async fn start_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
) -> CommandResult<()> {
    state.start_tunnel(tunnel_id).await.map_err(tunnel_error)
}

#[tauri::command]
pub async fn stop_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
) -> CommandResult<()> {
    state.stop_tunnel(tunnel_id).await.map_err(tunnel_error)
}

#[tauri::command]
pub async fn restart_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
) -> CommandResult<()> {
    state.restart_tunnel(tunnel_id).await.map_err(tunnel_error)
}

#[tauri::command]
pub async fn edit_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
    patch: UpdateTunnelRequest,
) -> CommandResult<()> {
    state
        .edit_tunnel(tunnel_id, patch)
        .await
        .map_err(tunnel_error)
}

#[tauri::command]
pub async fn delete_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
) -> CommandResult<()> {
    state.delete_tunnel(tunnel_id).await.map_err(tunnel_error)
}

fn tunnel_error(source: impl std::fmt::Display) -> AppError {
    AppError::from_source(
        "TUNNEL_OPERATION_FAILED",
        "errors.tunnel.operationFailed",
        source,
    )
}
