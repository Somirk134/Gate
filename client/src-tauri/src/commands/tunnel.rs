use tauri::State;

use crate::runtime::ClientRuntimeState;

#[tauri::command]
pub async fn create_tunnel(
    state: State<'_, ClientRuntimeState>,
    local_port: u16,
    remote_port: u16,
    protocol: String,
) -> Result<String, String> {
    state.create_tunnel(local_port, remote_port, protocol).await
}

#[tauri::command]
pub async fn delete_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
) -> Result<(), String> {
    state.delete_tunnel(tunnel_id).await
}
