use tauri::State;

#[tauri::command]
pub async fn connect(server_addr: String, token: String) -> Result<String, String> {
    todo!("connect to gate server")
}

#[tauri::command]
pub async fn disconnect() -> Result<(), String> {
    todo!("disconnect from gate server")
}
