use serde_json::Value;
use tauri::State;

use crate::commands::error::{AppError, CommandResult};
use crate::domain_center::{
    domain_batch, domain_bind_tunnel, domain_check_dns, domain_create, domain_delete, domain_detail,
    domain_list, domain_stats, domain_topology, domain_unbind_tunnel, DomainBatchRequest,
    DomainBindRequest, DomainCreateRequest, DomainListQuery,
};
use crate::project::ProjectWorkspaceState;
use crate::runtime::ClientRuntimeState;

#[tauri::command]
pub async fn domain_list_command(
    state: State<'_, ClientRuntimeState>,
    projects: State<'_, ProjectWorkspaceState>,
    query: DomainListQuery,
) -> CommandResult<Value> {
    domain_list(&state, &projects, query).await.map_err(domain_error)
}

#[tauri::command]
pub async fn domain_stats_command(
    state: State<'_, ClientRuntimeState>,
    projects: State<'_, ProjectWorkspaceState>,
) -> CommandResult<Value> {
    domain_stats(&state, &projects).await.map_err(domain_error)
}

#[tauri::command]
pub async fn domain_detail_command(
    state: State<'_, ClientRuntimeState>,
    projects: State<'_, ProjectWorkspaceState>,
    host: String,
) -> CommandResult<Value> {
    domain_detail(&state, &projects, host).await.map_err(domain_error)
}

#[tauri::command]
pub async fn domain_check_dns_command(
    state: State<'_, ClientRuntimeState>,
    host: String,
) -> CommandResult<Value> {
    domain_check_dns(&state, host).await.map_err(domain_error)
}

#[tauri::command]
pub async fn domain_create_command(
    state: State<'_, ClientRuntimeState>,
    projects: State<'_, ProjectWorkspaceState>,
    request: DomainCreateRequest,
) -> CommandResult<Value> {
    domain_create(&state, &projects, request)
        .await
        .map_err(domain_error)
}

#[tauri::command]
pub async fn domain_delete_command(
    state: State<'_, ClientRuntimeState>,
    host: String,
) -> CommandResult<Value> {
    domain_delete(&state, host).await.map_err(domain_error)
}

#[tauri::command]
pub async fn domain_bind_tunnel_command(
    state: State<'_, ClientRuntimeState>,
    request: DomainBindRequest,
) -> CommandResult<Value> {
    domain_bind_tunnel(&state, request).await.map_err(domain_error)
}

#[tauri::command]
pub async fn domain_unbind_tunnel_command(
    state: State<'_, ClientRuntimeState>,
    host: String,
) -> CommandResult<Value> {
    domain_unbind_tunnel(&state, host).await.map_err(domain_error)
}

#[tauri::command]
pub async fn domain_batch_command(
    state: State<'_, ClientRuntimeState>,
    projects: State<'_, ProjectWorkspaceState>,
    request: DomainBatchRequest,
) -> CommandResult<Value> {
    domain_batch(&state, &projects, request)
        .await
        .map_err(domain_error)
}

#[tauri::command]
pub async fn domain_topology_command(
    state: State<'_, ClientRuntimeState>,
    projects: State<'_, ProjectWorkspaceState>,
) -> CommandResult<Value> {
    domain_topology(&state, &projects).await.map_err(domain_error)
}

fn domain_error(source: impl std::fmt::Display) -> AppError {
    let message = source.to_string();
    if message == "DOMAIN_NOT_FOUND" {
        return AppError::new("DOMAIN_NOT_FOUND", "errors.domain.notFound");
    }
    if message == "DOMAIN_HOST_INVALID" {
        return AppError::new("DOMAIN_HOST_INVALID", "errors.domain.hostInvalid");
    }
    AppError::from_source("DOMAIN_OPERATION_FAILED", "errors.domain.operationFailed", message)
}
