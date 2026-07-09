use crate::gateway::{
    err, json_body, ok, read_message, response_for, write_message, TunnelGateway,
};
use chrono::Utc;
use gate_protocol::{Command, ProtocolBuilder};
use gate_shared::error::{AppError, NetworkError};
use gate_shared::lifecycle::ServerState;
use serde_json::{json, Value};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
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
        let token =
            std::env::var("GATE_AUTH_TOKEN").unwrap_or_else(|_| "gate-alpha-token".to_string());

        let listener = TcpListener::bind(addr).await.map_err(network_error)?;
        let bound_addr = listener.local_addr().map_err(network_error)?;
        let gateway = TunnelGateway::new();

        info!(
            target: "gate_server",
            addr = %bound_addr,
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
                            let token = token.clone();
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
    token: String,
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

                if relay_token != Some(token.as_str()) {
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
                if body.get("token").and_then(Value::as_str) == Some(token.as_str()) {
                    authenticated = true;
                    let id = gateway.create_session().await;
                    session_id = Some(id.clone());
                    write_message(
                        &mut stream,
                        &protocol,
                        &response_for(&message, ok(json!({ "sessionId": id }))),
                    )
                    .await?;
                    info!(target: "gate_server", "Session Create");
                    info!(target: "gate_server", "Server Running");
                } else {
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
                if let Some(session_id) = session_id.as_deref() {
                    gateway.touch_session(session_id).await;
                }
                write_message(
                    &mut stream,
                    &protocol,
                    &response_for(
                        &message,
                        ok(json!({
                            "kind": "pong",
                            "sessionId": &session_id,
                            "timestamp": Utc::now().timestamp_millis()
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
