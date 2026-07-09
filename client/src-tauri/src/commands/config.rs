use std::{fs, path::Path};

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

#[tauri::command]
pub async fn export_config_file(path: String, content: String) -> Result<String, String> {
    if path.trim().is_empty() {
        return Err("导出路径不能为空".to_string());
    }

    let destination = Path::new(&path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    // 由后端写入用户在保存对话框中选择的路径，避免放宽前端 FS 插件权限。
    fs::write(destination, content).map_err(|error| error.to_string())?;
    Ok(destination.to_string_lossy().to_string())
}
