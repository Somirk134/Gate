#[tauri::command]
pub async fn create_tunnel(
    local_port: u16,
    remote_port: u16,
    protocol: String,
) -> Result<String, String> {
    todo!("create tunnel")
}

#[tauri::command]
pub async fn delete_tunnel(tunnel_id: String) -> Result<(), String> {
    todo!("delete tunnel")
}
