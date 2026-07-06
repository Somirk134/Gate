use chrono::Utc;
use gate_protocol::{
    Body, Command, Frame, FrameEncoder, Message, MessageType, Metadata, ProtocolBuilder,
    ProtocolManager,
};
use gate_shared::error::{AppError, NetworkError};
use gate_shared::lifecycle::ServerState;
use serde_json::{json, Value};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tracing::{error, info, warn};
use uuid::Uuid;

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
        let token = std::env::var("GATE_AUTH_TOKEN")
            .unwrap_or_else(|_| "gate-alpha-token".to_string());

        let listener = TcpListener::bind(addr).await.map_err(network_error)?;
        let bound_addr = listener.local_addr().map_err(network_error)?;

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
                            tokio::spawn(async move {
                                if let Err(source) = handle_connection(stream, token).await {
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

async fn handle_connection(mut stream: TcpStream, token: String) -> anyhow::Result<()> {
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
            Command::AuthLogin => {
                info!(target: "gate_server", "Authenticate");
                let body = json_body(&message);
                if body.get("token").and_then(Value::as_str) == Some(token.as_str()) {
                    authenticated = true;
                    let id = Uuid::new_v4().to_string();
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
            Command::StatisticsQuery if authenticated => {
                write_message(
                    &mut stream,
                    &protocol,
                    &response_for(
                        &message,
                        ok(json!({
                            "requestTotal": request_total,
                            "heartbeatTotal": heartbeat_total,
                            "authenticated": authenticated
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

    Ok(())
}

async fn read_message(
    stream: &mut TcpStream,
    protocol: &ProtocolManager,
) -> anyhow::Result<Option<Message>> {
    let mut length = [0_u8; 4];
    match stream.read_exact(&mut length).await {
        Ok(_) => {}
        Err(source) if source.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(source) => return Err(source.into()),
    }

    let length = u32::from_be_bytes(length) as usize;
    let mut payload = vec![0_u8; length];
    stream.read_exact(&mut payload).await?;
    Ok(Some(protocol.decode(&payload)?))
}

async fn write_message(
    stream: &mut TcpStream,
    protocol: &ProtocolManager,
    message: &Message,
) -> anyhow::Result<()> {
    let payload = protocol.encode(message)?;
    let frame = Frame::new(payload)?;
    let bytes = FrameEncoder::encode(&frame);
    stream.write_all(&bytes).await?;
    stream.flush().await?;
    Ok(())
}

fn response_for(request: &Message, body: Value) -> Message {
    let mut response = Message::new(
        MessageType::Response,
        request.header.command.clone(),
        Body::Json(body),
        Metadata::default(),
    );
    response.header.request_id = request.header.request_id;
    response
}

fn json_body(message: &Message) -> Value {
    match &message.body {
        Body::Json(value) => value.clone(),
        _ => Value::Null,
    }
}

fn ok(data: Value) -> Value {
    json!({ "ok": true, "data": data })
}

fn err(code: &str, message: &str) -> Value {
    json!({
        "ok": false,
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn network_error(source: std::io::Error) -> AppError {
    NetworkError::ComponentFailure {
        message: source.to_string(),
    }
    .into()
}
