use std::{fs, path::Path};

use serde_json::Value;
use tauri::State;

use crate::commands::error::{AppError, CommandResult};
use crate::runtime::ClientRuntimeState;

#[tauri::command]
pub async fn get_config(state: State<'_, ClientRuntimeState>) -> CommandResult<Value> {
    Ok(state.config().await)
}

#[tauri::command]
pub async fn set_config(
    state: State<'_, ClientRuntimeState>,
    key: String,
    value: String,
) -> CommandResult<()> {
    state.set_config(key, value).await.map_err(|source| {
        AppError::from_source(
            "RUNTIME_CONFIG_WRITE_FAILED",
            "errors.runtime.configWriteFailed",
            source,
        )
    })
}

#[tauri::command]
pub async fn export_config_file(path: String, content: String) -> CommandResult<String> {
    if path.trim().is_empty() {
        return Err(AppError::new(
            "CONFIG_EXPORT_PATH_EMPTY",
            "errors.config.exportPathEmpty",
        ));
    }

    let destination = Path::new(&path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::from_source(
                "CONFIG_EXPORT_DIRECTORY_CREATE_FAILED",
                "errors.config.directoryCreateFailed",
                error,
            )
        })?;
    }

    // 由后端写入用户选择的导出路径，避免放宽前端 FS 插件权限。
    fs::write(destination, content).map_err(|error| {
        AppError::from_source(
            "CONFIG_EXPORT_WRITE_FAILED",
            "errors.config.writeFailed",
            error,
        )
    })?;
    Ok(destination.to_string_lossy().to_string())
}
