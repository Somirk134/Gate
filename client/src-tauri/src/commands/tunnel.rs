use serde_json::json;
use tauri::State;

use crate::commands::error::{AppError, CommandResult};
use crate::runtime::{ClientRuntimeState, UpdateTunnelRequest};
use crate::tunnel_performance::TunnelPerformanceRecommendation;

#[tauri::command]
pub async fn tunnel_recommend_performance(
    state: State<'_, ClientRuntimeState>,
    mode: Option<String>,
) -> CommandResult<TunnelPerformanceRecommendation> {
    Ok(state.recommend_tunnel_performance(mode).await)
}

#[tauri::command]
pub async fn create_tunnel(
    state: State<'_, ClientRuntimeState>,
    local_port: u16,
    remote_port: u16,
    protocol: String,
    server_id: Option<String>,
    local_host: Option<String>,
    host: Option<String>,
    path: Option<String>,
) -> CommandResult<String> {
    state
        .create_tunnel(
            local_port,
            remote_port,
            protocol,
            server_id,
            local_host,
            host,
            path,
        )
        .await
        .map_err(tunnel_error)
}

#[tauri::command]
pub async fn start_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
) -> CommandResult<()> {
    state.start_tunnel(tunnel_id).await.map_err(tunnel_error)
}

#[tauri::command]
pub async fn stop_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
) -> CommandResult<()> {
    state.stop_tunnel(tunnel_id).await.map_err(tunnel_error)
}

#[tauri::command]
pub async fn restart_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
) -> CommandResult<()> {
    state.restart_tunnel(tunnel_id).await.map_err(tunnel_error)
}

#[tauri::command]
pub async fn edit_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
    patch: UpdateTunnelRequest,
) -> CommandResult<()> {
    state
        .edit_tunnel(tunnel_id, patch)
        .await
        .map_err(tunnel_error)
}

#[tauri::command]
pub async fn delete_tunnel(
    state: State<'_, ClientRuntimeState>,
    tunnel_id: String,
) -> CommandResult<()> {
    state.delete_tunnel(tunnel_id).await.map_err(tunnel_error)
}

fn tunnel_error(source: impl std::fmt::Display) -> AppError {
    classify_tunnel_error(&source.to_string())
}

fn classify_tunnel_error(source: &str) -> AppError {
    let lower = source.to_ascii_lowercase();

    if source == "tunnel not found" {
        return AppError::new("TUNNEL_NOT_FOUND", "tunnel.errors.notFound");
    }
    if source == "SERVER_SESSION_MISSING" {
        return AppError::new("TUNNEL_SESSION_MISSING", "tunnel.errors.sessionExpired");
    }
    if source == "NO_AVAILABLE_SERVER_CONFIG" {
        return AppError::new("TUNNEL_NO_SERVER", "tunnel.errors.noServer");
    }
    if source == "SERVER_NOT_FOUND_FOR_TUNNEL" || source == "SERVER_REQUIRED_FOR_TUNNEL" {
        return AppError::new("TUNNEL_CONFIG_INCOMPLETE", "tunnel.errors.configServerMissing");
    }
    if source.starts_with("REMOTE_PORT_OCCUPIED:") {
        let port = source.rsplit(':').next().unwrap_or_default();
        return AppError::with_details(
            "TUNNEL_REMOTE_PORT_OCCUPIED",
            "tunnel.errors.remotePortOccupied",
            json!({ "port": port, "source": source }),
        );
    }
    if source.contains("REMOTE_PORT_REQUIRED") {
        return AppError::new("TUNNEL_REMOTE_PORT_REQUIRED", "tunnel.errors.remotePortRequired");
    }
    if source.contains("LOCAL_PORT_REQUIRED") {
        return AppError::new("TUNNEL_LOCAL_PORT_REQUIRED", "tunnel.errors.localPortRequired");
    }
    if source.contains("SERVER_DISCONNECTED_TUNNEL_START_BLOCKED")
        || lower.contains("no active connection")
        || lower.contains("server is disconnected")
        || lower.contains("runtime backend is not connected")
    {
        return AppError::with_details(
            "TUNNEL_SERVER_OFFLINE",
            "tunnel.errors.serverOffline",
            json!({ "source": source }),
        );
    }
    if lower.contains("local service")
        && (lower.contains("unreachable")
            || lower.contains("timed out")
            || lower.contains("refused"))
    {
        return AppError::with_details(
            "TUNNEL_LOCAL_SERVICE_UNREACHABLE",
            "tunnel.errors.localServiceUnreachableGeneric",
            json!({ "source": source }),
        );
    }
    if lower.contains("failed to resolve local service address") || lower.contains("address is invalid")
    {
        return AppError::with_details(
            "TUNNEL_LOCAL_SERVICE_INVALID",
            "tunnel.errors.localServiceInvalidGeneric",
            json!({ "source": source }),
        );
    }
    if lower.contains("already in use") || lower.contains("10048") || lower.contains("only one usage")
    {
        return AppError::with_details(
            "TUNNEL_LOCAL_PORT_OCCUPIED",
            "tunnel.errors.localPortOccupied",
            json!({ "source": source }),
        );
    }
    if lower.contains("token") || lower.contains("auth") {
        return AppError::with_details(
            "TUNNEL_AUTH_FAILED",
            "tunnel.errors.authFailed",
            json!({ "source": source }),
        );
    }
    if lower.contains("certificate") || lower.contains("acme") {
        return AppError::with_details(
            "TUNNEL_CERTIFICATE_FAILED",
            "tunnel.errors.certificateFailed",
            json!({ "source": source }),
        );
    }
    if lower.contains("permission denied") || lower.contains("os error 13") {
        return AppError::with_details(
            "TUNNEL_BIND_PERMISSION_DENIED",
            "tunnel.errors.bindPermissionDenied",
            json!({ "source": source }),
        );
    }
    if lower.contains("already bound to tunnel") {
        return AppError::with_details(
            "TUNNEL_DOMAIN_ALREADY_BOUND",
            "tunnel.errors.domainAlreadyBound",
            json!({ "source": source }),
        );
    }
    if lower.contains("server unavailable") || lower.contains("control connection disconnected") {
        return AppError::with_details(
            "TUNNEL_SERVER_CONTROL_FAILED",
            "tunnel.errors.serverControlFailed",
            json!({ "source": source }),
        );
    }
    if lower.contains("local runtime failed to start") {
        return AppError::with_details(
            "TUNNEL_LOCAL_RUNTIME_FAILED",
            "tunnel.errors.localRuntimeFailedGeneric",
            json!({ "source": source }),
        );
    }

    AppError::from_source(
        "TUNNEL_OPERATION_FAILED",
        "tunnel.errors.operationFailedDetail",
        source,
    )
}
