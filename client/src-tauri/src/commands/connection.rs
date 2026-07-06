use tauri::State;

use crate::runtime::ClientRuntimeState;

#[tauri::command]
pub async fn connect(
    state: State<'_, ClientRuntimeState>,
    server_addr: String,
    token: String,
) -> Result<String, String> {
    state.connect(server_addr, token).await
}

#[tauri::command]
pub async fn disconnect(state: State<'_, ClientRuntimeState>) -> Result<(), String> {
    state.disconnect().await
}

#[tauri::command]
pub async fn heartbeat(state: State<'_, ClientRuntimeState>) -> Result<u64, String> {
    state.heartbeat().await
}
