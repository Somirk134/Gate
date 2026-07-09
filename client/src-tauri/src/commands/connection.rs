use tauri::State;

use crate::commands::error::{AppError, CommandResult};
use crate::runtime::ClientRuntimeState;

#[tauri::command]
pub async fn connect(
    state: State<'_, ClientRuntimeState>,
    server_addr: String,
    token: String,
) -> CommandResult<String> {
    state
        .connect(server_addr, token)
        .await
        .map_err(connection_error)
}

#[tauri::command]
pub async fn disconnect(state: State<'_, ClientRuntimeState>) -> CommandResult<()> {
    state.disconnect().await.map_err(connection_error)
}

#[tauri::command]
pub async fn heartbeat(state: State<'_, ClientRuntimeState>) -> CommandResult<u64> {
    state.heartbeat().await.map_err(connection_error)
}

fn connection_error(source: impl std::fmt::Display) -> AppError {
    AppError::from_source(
        "CONNECTION_OPERATION_FAILED",
        "errors.connection.operationFailed",
        source,
    )
}
