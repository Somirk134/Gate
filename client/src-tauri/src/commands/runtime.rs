use crate::runtime::ClientRuntimeState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub async fn runtime_get_statistics(state: State<'_, ClientRuntimeState>) -> Result<Value, String> {
    Ok(state.statistics().await)
}

#[tauri::command]
pub async fn runtime_get_dashboard(state: State<'_, ClientRuntimeState>) -> Result<Value, String> {
    Ok(state.dashboard().await)
}

#[tauri::command]
pub async fn runtime_get_health(state: State<'_, ClientRuntimeState>) -> Result<Value, String> {
    Ok(state.health().await)
}

#[tauri::command]
pub async fn runtime_collect_metrics(
    state: State<'_, ClientRuntimeState>,
) -> Result<Value, String> {
    Ok(state.metrics().await)
}

#[tauri::command]
pub async fn runtime_get_logs(state: State<'_, ClientRuntimeState>) -> Result<Value, String> {
    Ok(state.logs().await)
}

#[tauri::command]
pub async fn runtime_get_store_report(
    state: State<'_, ClientRuntimeState>,
) -> Result<Value, String> {
    Ok(state.runtime_store_report().await)
}

#[tauri::command]
pub async fn runtime_run_startup_diagnostics(
    state: State<'_, ClientRuntimeState>,
) -> Result<Value, String> {
    Ok(state.startup_diagnostics().await)
}
