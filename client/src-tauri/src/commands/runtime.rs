use crate::commands::error::{AppError, CommandResult};
use crate::runtime::ClientRuntimeState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub async fn runtime_get_statistics(state: State<'_, ClientRuntimeState>) -> CommandResult<Value> {
    Ok(state.statistics().await)
}

#[tauri::command]
pub async fn runtime_get_dashboard(state: State<'_, ClientRuntimeState>) -> CommandResult<Value> {
    Ok(state.dashboard().await)
}

#[tauri::command]
pub async fn runtime_get_health(state: State<'_, ClientRuntimeState>) -> CommandResult<Value> {
    Ok(state.health().await)
}

#[tauri::command]
pub async fn runtime_collect_metrics(state: State<'_, ClientRuntimeState>) -> CommandResult<Value> {
    Ok(state.metrics().await)
}

#[tauri::command]
pub async fn runtime_get_logs(state: State<'_, ClientRuntimeState>) -> CommandResult<Value> {
    Ok(state.logs().await)
}

#[tauri::command]
pub async fn runtime_clear_logs(state: State<'_, ClientRuntimeState>) -> CommandResult<()> {
    state.clear_logs().await.map_err(runtime_error)
}

fn runtime_error(source: impl std::fmt::Display) -> AppError {
    AppError::from_source(
        "RUNTIME_OPERATION_FAILED",
        "errors.runtime.operationFailed",
        source,
    )
}
