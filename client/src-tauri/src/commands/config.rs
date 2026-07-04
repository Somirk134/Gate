#[tauri::command]
pub async fn get_config() -> Result<String, String> {
    todo!("get application config")
}

#[tauri::command]
pub async fn set_config(key: String, value: String) -> Result<(), String> {
    todo!("set application config")
}
