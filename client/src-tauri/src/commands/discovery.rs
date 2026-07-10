use serde_json::Value;
use tauri::State;

use crate::{commands::error::CommandResult, runtime::ClientRuntimeState};

#[tauri::command]
pub async fn discovery_local_services(
    state: State<'_, ClientRuntimeState>,
) -> CommandResult<Value> {
    Ok(state.local_services().await)
}

#[tauri::command]
pub async fn discovery_probe_local_service(
    state: State<'_, ClientRuntimeState>,
    host: String,
    port: u16,
) -> CommandResult<Value> {
    Ok(state.probe_local_service(host, port).await)
}

#[tauri::command]
pub async fn discovery_remote_ports(
    state: State<'_, ClientRuntimeState>,
    server_id: Option<String>,
) -> CommandResult<Value> {
    Ok(state.remote_port_discovery(server_id).await)
}

#[tauri::command]
pub async fn discovery_check_remote_port(
    state: State<'_, ClientRuntimeState>,
    server_id: Option<String>,
    port: u16,
) -> CommandResult<Value> {
    Ok(state.check_remote_port(server_id, port).await)
}

#[tauri::command]
pub async fn discovery_diagnose_tunnel(
    state: State<'_, ClientRuntimeState>,
    local_host: String,
    local_port: u16,
    remote_port: u16,
    server_id: Option<String>,
) -> CommandResult<Value> {
    Ok(state
        .diagnose_tunnel(local_host, local_port, remote_port, server_id)
        .await)
}
