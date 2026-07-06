use serde_json::Value;
use tauri::State;

use crate::runtime::ClientRuntimeState;

#[tauri::command]
pub async fn get_config(state: State<'_, ClientRuntimeState>) -> Result<Value, String> {
    Ok(state.config().await)
}

#[tauri::command]
pub async fn set_config(
    state: State<'_, ClientRuntimeState>,
    key: String,
    value: String,
) -> Result<(), String> {
    state.set_config(key, value).await
}
