use serde_json::Value;
use tauri::State;

use crate::runtime::{ClientRuntimeState, CreateServerRequest, UpdateServerRequest};

#[tauri::command]
pub async fn server_list(state: State<'_, ClientRuntimeState>) -> Result<Value, String> {
    Ok(state.list_servers().await)
}

#[tauri::command]
pub async fn server_create(
    state: State<'_, ClientRuntimeState>,
    request: CreateServerRequest,
) -> Result<String, String> {
    state.create_server(request).await
}

#[tauri::command]
pub async fn server_update(
    state: State<'_, ClientRuntimeState>,
    server_id: String,
    patch: UpdateServerRequest,
) -> Result<(), String> {
    state.update_server(server_id, patch).await
}

#[tauri::command]
pub async fn server_delete(
    state: State<'_, ClientRuntimeState>,
    server_id: String,
) -> Result<(), String> {
    state.delete_server(server_id).await
}

#[tauri::command]
pub async fn server_connect(
    state: State<'_, ClientRuntimeState>,
    server_id: String,
) -> Result<String, String> {
    state.connect_server(server_id).await
}

#[tauri::command]
pub async fn server_disconnect(
    state: State<'_, ClientRuntimeState>,
    server_id: String,
) -> Result<(), String> {
    state.disconnect_server(server_id).await
}

#[tauri::command]
pub async fn server_test(
    state: State<'_, ClientRuntimeState>,
    server_id: String,
) -> Result<Value, String> {
    state.test_server(server_id).await
}
