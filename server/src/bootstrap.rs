use crate::gateway::{
    err, json_body, ok, read_message, response_for, write_message, TunnelGateway,
};
use chrono::Utc;
use gate_protocol::{Command, ProtocolBuilder};
use gate_shared::error::{AppError, ConfigError, NetworkError};
use gate_shared::lifecycle::ServerState;
use local_ip_address::local_ip;
use serde_json::{json, Value};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerBootstrap {
    initial_state: ServerState,
}

impl Default for ServerBootstrap {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerBootstrap {
    pub fn new() -> Self {
        Self {
            initial_state: ServerState::Starting,
        }
    }

    pub fn initial_state(&self) -> ServerState {
        self.initial_state
    }

    pub async fn boot(self) -> Result<(), AppError> {
        let addr = std::env::var("GATE_SERVER_ADDR")
            .ok()
            .and_then(|value| value.parse::<SocketAddr>().ok())
            .unwrap_or_else(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 7000));
        // 鉴权凭据属于生产启动前置条件，禁止使用缺省值或已知弱值继续启动。
        let token = load_auth_token()?;

        let listener = TcpListener::bind(addr).await.map_err(network_error)?;
        let bound_addr = listener.local_addr().map_err(network_error)?;
        let gateway = TunnelGateway::new();

        let local_ip_str = local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        let access_addr = SocketAddr::new(
            local_ip().map(|ip| ip).unwrap_or_else(|_| bound_addr.ip()),
            bound_addr.port(),
        );

        println!();
        println!("========================================");
        println!("  Gate Server started successfully");
        println!("========================================");
        println!("  Listen address : {}", bound_addr);
        println!("  Local IP       : {}", local_ip_str);
        println!("  Access address : {}", access_addr);
        println!("  Authentication : configured");
        println!("========================================");
        println!();
        println!("Use this info to connect Gate client.");
        println!();

        info!(
            target: "gate_server",
            addr = %bound_addr,
            local_ip = %local_ip_str,
            "Server Boot"
        );
        info!(
            target: "gate_server",
            addr = %bound_addr,
            "Server Listen"
        );

        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    info!(target: "gate_server", "Server Shutdown");
                    gateway.shutdown().await;
                    break;
                }
                accepted = listener.accept() => {
                    match accepted {
                        Ok((stream, remote_addr)) => {
                            info!(
                                target: "gate_server",
                                remote_addr = %remote_addr,
                                "Server Accept"
                            );
                            let token = Arc::clone(&token);
                            let gateway = gateway.clone();
                            tokio::spawn(async move {
                                if let Err(source) = handle_connection(stream, token, gateway).await {
                                    error!(
                                        target: "gate_server",
                                        error = %source,
                                        "connection failed"
                                    );
                                }
                            });
                        }
                        Err(source) => {
                            warn!(
                                target: "gate_server",
                                error = %source,
                                "accept failed"
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    token: Arc<str>,
    gateway: TunnelGateway,
) -> anyhow::Result<()> {
    let protocol = ProtocolBuilder::new().build();
    let mut authenticated = false;
    let mut session_id: Option<String> = None;
    let mut request_total = 0_u64;
    let mut heartbeat_total = 0_u64;

    loop {
        let Some(message) = read_message(&mut stream, &protocol).await? else {
            break;
        };
        request_total += 1;

        match message.header.command {
            Command::TunnelRelayConnect => {
                let body = json_body(&message);
                let relay_token = body.get("token").and_then(Value::as_str);
                let relay_session_id = body.get("sessionId").and_then(Value::as_str);
                let relay_tunnel_id = body
                    .get("tunnelId")
                    .or_else(|| body.get("id"))
                    .and_then(Value::as_str);

                let (Some(relay_session_id), Some(relay_tunnel_id)) =
                    (relay_session_id, relay_tunnel_id)
                else {
                    write_message(
                        &mut stream,
                        &protocol,
                        &response_for(
                            &message,
                            err("RELAY_INVALID", "relay sessionId and tunnelId are required"),
                        ),
                    )
                    .await?;
                    break;
                };

                if relay_token != Some(token.as_ref()) {
                    gateway.record_session_failure().await;
                    write_message(
                        &mut stream,
                        &protocol,
                        &response_for(&message, err("AUTH_FAILED", "invalid token")),
                    )
                    .await?;
                    break;
                }

                if let Err(source) = gateway
                    .validate_relay_worker(relay_tunnel_id, relay_session_id)
                    .await
                {
                    write_message(
                        &mut stream,
                        &protocol,
                        &response_for(&message, err("RELAY_REJECTED", &source.to_string())),
                    )
                    .await?;
                    break;
                }

                write_message(
                    &mut stream,
                    &protocol,
                    &response_for(
                        &message,
                        ok(json!({
                            "waiting": true,
                            "sessionId": relay_session_id,
                            "tunnelId": relay_tunnel_id
                        })),
                    ),
                )
                .await?;
                gateway
                    .attach_relay_worker(relay_tunnel_id, relay_session_id, stream)
                    .await?;
                return Ok(());
            }
            Command::AuthLogin => {
                info!(target: "gate_server", "Authenticate");
                let body = json_body(&message);
                if body.get("token").and_then(Value::as_str) == Some(token.as_ref()) {
                    authenticated = true;
                    let client_id = body
                        .get("clientId")
                        .or_else(|| body.get("client_id"))
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned);
                    let requested_session_id = body
                        .get("sessionId")
                        .or_else(|| body.get("session_id"))
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned);
                    let id = gateway
                        .create_or_restore_session(client_id, requested_session_id)
                        .await;
                    session_id = Some(id.clone());
                    let server = gateway.capability_snapshot().await;
                    write_message(
                        &mut stream,
                        &protocol,
                        &response_for(&message, ok(json!({ "sessionId": id, "server": server }))),
                    )
                    .await?;
                    info!(target: "gate_server", "Session Create");
                    info!(target: "gate_server", "Server Running");
                } else {
                    gateway.record_session_failure().await;
                    write_message(
                        &mut stream,
                        &protocol,
                        &response_for(&message, err("AUTH_FAILED", "invalid token")),
                    )
                    .await?;
                    break;
                }
            }
            Command::HeartbeatPing if authenticated => {
                heartbeat_total += 1;
                let heartbeat = if let Some(session_id) = session_id.as_deref() {
                    gateway
                        .update_heartbeat(session_id, &json_body(&message))
                        .await
                } else {
                    json!({})
                };
                let server = gateway.capability_snapshot().await;
                write_message(
                    &mut stream,
                    &protocol,
                    &response_for(
                        &message,
                        ok(json!({
                            "kind": "pong",
                            "sessionId": &session_id,
                            "timestamp": Utc::now().timestamp_millis(),
                            "heartbeat": heartbeat,
                            "server": server
                        })),
                    ),
                )
                .await?;
            }
            Command::TunnelCreate | Command::TunnelRegister | Command::TunnelStart
                if authenticated =>
            {
                let Some(session_id) = session_id.as_deref() else {
                    write_message(
                        &mut stream,
                        &protocol,
                        &response_for(&message, err("SESSION_REQUIRED", "session is missing")),
                    )
                    .await?;
                    continue;
                };

                match gateway
                    .register_tunnel(session_id, &json_body(&message))
                    .await
                {
                    Ok(data) => {
                        write_message(&mut stream, &protocol, &response_for(&message, ok(data)))
                            .await?;
                    }
                    Err(source) => {
                        write_message(
                            &mut stream,
                            &protocol,
                            &response_for(
                                &message,
                                err("TUNNEL_REGISTER_FAILED", &source.to_string()),
                            ),
                        )
                        .await?;
                    }
                }
            }
            Command::TunnelStop if authenticated => {
                let body = json_body(&message);
                let tunnel_id = body
                    .get("id")
                    .or_else(|| body.get("tunnelId"))
                    .and_then(Value::as_str);
                let Some(tunnel_id) = tunnel_id else {
                    write_message(
                        &mut stream,
                        &protocol,
                        &response_for(&message, err("TUNNEL_INVALID", "tunnel id is required")),
                    )
                    .await?;
                    continue;
                };

                let stopped = gateway.stop_tunnel(tunnel_id).await?;
                write_message(
                    &mut stream,
                    &protocol,
                    &response_for(
                        &message,
                        ok(json!({
                            "tunnelId": tunnel_id,
                            "stopped": stopped
                        })),
                    ),
                )
                .await?;
            }
            Command::DomainCreate if authenticated => match gateway
                .create_domain(&json_body(&message))
                .await
            {
                Ok(data) => {
                    write_message(&mut stream, &protocol, &response_for(&message, ok(data)))
                        .await?;
                }
                Err(source) => {
                    write_message(
                        &mut stream,
                        &protocol,
                        &response_for(&message, err("DOMAIN_CREATE_FAILED", &source.to_string())),
                    )
                    .await?;
                }
            },
            Command::DomainBind if authenticated => {
                match gateway.bind_domain(&json_body(&message)).await {
                    Ok(data) => {
                        write_message(&mut stream, &protocol, &response_for(&message, ok(data)))
                            .await?;
                    }
                    Err(source) => {
                        write_message(
                            &mut stream,
                            &protocol,
                            &response_for(&message, err("DOMAIN_BIND_FAILED", &source.to_string())),
                        )
                        .await?;
                    }
                }
            }
            Command::DomainUnbind if authenticated => {
                match gateway.unbind_domain(&json_body(&message)).await {
                    Ok(data) => {
                        write_message(&mut stream, &protocol, &response_for(&message, ok(data)))
                            .await?;
                    }
                    Err(source) => {
                        write_message(
                            &mut stream,
                            &protocol,
                            &response_for(
                                &message,
                                err("DOMAIN_UNBIND_FAILED", &source.to_string()),
                            ),
                        )
                        .await?;
                    }
                }
            }
            Command::DomainDelete if authenticated => {
                match gateway.delete_domain(&json_body(&message)).await {
                    Ok(data) => {
                        write_message(&mut stream, &protocol, &response_for(&message, ok(data)))
                            .await?;
                    }
                    Err(source) => {
                        write_message(
                            &mut stream,
                            &protocol,
                            &response_for(
                                &message,
                                err("DOMAIN_DELETE_FAILED", &source.to_string()),
                            ),
                        )
                        .await?;
                    }
                }
            }
            Command::StatisticsQuery if authenticated => {
                let gateway_statistics = gateway.statistics().await;
                write_message(
                    &mut stream,
                    &protocol,
                    &response_for(
                        &message,
                        ok(json!({
                            "requestTotal": request_total,
                            "heartbeatTotal": heartbeat_total,
                            "authenticated": authenticated,
                            "gateway": gateway_statistics
                        })),
                    ),
                )
                .await?;
            }
            Command::SystemShutdown if authenticated => {
                write_message(
                    &mut stream,
                    &protocol,
                    &response_for(&message, ok(json!({ "shutdown": true }))),
                )
                .await?;
                break;
            }
            _ if !authenticated => {
                write_message(
                    &mut stream,
                    &protocol,
                    &response_for(&message, err("AUTH_REQUIRED", "authenticate first")),
                )
                .await?;
                break;
            }
            _ => {
                write_message(
                    &mut stream,
                    &protocol,
                    &response_for(&message, ok(json!({ "accepted": true }))),
                )
                .await?;
            }
        }
    }

    if let Some(session_id) = session_id {
        gateway.close_session(&session_id).await;
    }

    Ok(())
}

fn network_error(source: std::io::Error) -> AppError {
    NetworkError::ComponentFailure {
        message: source.to_string(),
    }
    .into()
}

fn load_auth_token() -> Result<Arc<str>, AppError> {
    let token = std::env::var("GATE_AUTH_TOKEN").map_err(|_| ConfigError::Validation {
        key: "GATE_AUTH_TOKEN".to_string(),
        message: "authentication token is required".to_string(),
    })?;
    let token = token.trim();
    let known_weak = matches!(
        token.to_ascii_lowercase().as_str(),
        "gate-alpha-token"
            | "change-me"
            | "changeme"
            | "replace-me"
            | "replace-with-a-long-random-token"
    );
    if token.len() < 16 || known_weak {
        return Err(ConfigError::Validation {
            key: "GATE_AUTH_TOKEN".to_string(),
            message: "authentication token must contain at least 16 characters and must not use a known default".to_string(),
        }
        .into());
    }

    Ok(Arc::from(token))
}
