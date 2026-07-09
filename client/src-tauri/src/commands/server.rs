use serde_json::Value;
use tauri::State;

use crate::commands::error::{AppError, CommandResult};
use crate::runtime::{ClientRuntimeState, CreateServerRequest, UpdateServerRequest};

#[tauri::command]
pub async fn server_list(state: State<'_, ClientRuntimeState>) -> CommandResult<Value> {
    Ok(state.list_servers().await)
}

#[tauri::command]
pub async fn server_create(
    state: State<'_, ClientRuntimeState>,
    request: CreateServerRequest,
) -> CommandResult<String> {
    state.create_server(request).await.map_err(server_error)
}

#[tauri::command]
pub async fn server_update(
    state: State<'_, ClientRuntimeState>,
    server_id: String,
    patch: UpdateServerRequest,
) -> CommandResult<()> {
    state
        .update_server(server_id, patch)
        .await
        .map_err(server_error)
}

#[tauri::command]
pub async fn server_delete(
    state: State<'_, ClientRuntimeState>,
    server_id: String,
) -> CommandResult<()> {
    state.delete_server(server_id).await.map_err(server_error)
}

#[tauri::command]
pub async fn server_connect(
    state: State<'_, ClientRuntimeState>,
    server_id: String,
) -> CommandResult<String> {
    state.connect_server(server_id).await.map_err(server_error)
}

#[tauri::command]
pub async fn server_disconnect(
    state: State<'_, ClientRuntimeState>,
    server_id: String,
) -> CommandResult<()> {
    state
        .disconnect_server(server_id)
        .await
        .map_err(server_error)
}

#[tauri::command]
pub async fn server_test(
    state: State<'_, ClientRuntimeState>,
    server_id: String,
) -> CommandResult<Value> {
    state.test_server(server_id).await.map_err(server_error)
}

fn server_error(source: impl std::fmt::Display) -> AppError {
    AppError::from_source(
        "SERVER_OPERATION_FAILED",
        "errors.server.operationFailed",
        source,
    )
}
