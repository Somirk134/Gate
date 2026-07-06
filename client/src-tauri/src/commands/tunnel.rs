use tauri::State;

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
) -> Result<String, String> {
    state
        .create_tunnel(local_port, remote_port, protocol, local_host, host, path)
        .await
}

#[tauri::command]
pub async fn start_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
) -> Result<(), String> {
    state.start_tunnel(tunnel_id).await
}

#[tauri::command]
pub async fn stop_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
) -> Result<(), String> {
    state.stop_tunnel(tunnel_id).await
}

#[tauri::command]
pub async fn restart_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
) -> Result<(), String> {
    state.restart_tunnel(tunnel_id).await
}

#[tauri::command]
pub async fn edit_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
    patch: UpdateTunnelRequest,
) -> Result<(), String> {
    state.edit_tunnel(tunnel_id, patch).await
}

#[tauri::command]
pub async fn delete_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
) -> Result<(), String> {
    state.delete_tunnel(tunnel_id).await
}
