use crate::{commands::error::CommandResult, runtime::ClientRuntimeState};
use gate_communication::{TcpTransport, Transport, TransportEndpoint};
use gate_protocol::{Body, Command, Message, Metadata};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    env, fs,
    net::SocketAddr,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::State;
use tokio::{net::TcpStream, time::timeout};

const DIAGNOSTIC_VALUE_DISCONNECTED: &str = "DIAGNOSTIC_VALUE_DISCONNECTED";
const DIAGNOSTIC_VALUE_MEMORY_PERMISSION_REQUIRED: &str =
    "DIAGNOSTIC_VALUE_MEMORY_PERMISSION_REQUIRED";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticAction {
    label: String,
    description: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticFinding {
    id: String,
    label: String,
    status: String,
    reason: String,
    possible_cause: String,
    solution: String,
    elapsed_ms: Option<u128>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestReport {
    ok: bool,
    code: String,
    title: String,
    reason: String,
    possible_cause: String,
    solution: String,
    elapsed_ms: u128,
    checked_at: i64,
    actions: Vec<DiagnosticAction>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentCheckReport {
    ok: bool,
    checked_at: i64,
    summary: String,
    findings: Vec<DiagnosticFinding>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfoReport {
    client_version: String,
    server_version: String,
    protocol_version: String,
    rust_version: String,
    os: String,
    arch: String,
    cpu: String,
    memory: String,
    config_dir: String,
    log_dir: String,
    current_dir: String,
}

#[tauri::command]
pub async fn diagnostics_test_connection(
    server_addr: String,
    token: String,
    timeout_ms: Option<u64>,
) -> CommandResult<ConnectionTestReport> {
    Ok(test_connection(server_addr, token, timeout_ms.unwrap_or(5000)).await)
}

#[tauri::command]
pub async fn diagnostics_run_deployment(
    state: State<'_, ClientRuntimeState>,
    server_addr: Option<String>,
) -> CommandResult<DeploymentCheckReport> {
    let checked_at = now_ms();
    let mut findings = Vec::new();

    findings.push(check_current_exe());
    findings.push(check_current_dir());
    findings.push(check_config_file());
    findings.push(check_log_dir());
    findings.push(check_temp_permission());
    findings.push(check_runtime_config(state).await);

    if let Some(addr) = server_addr.filter(|value| !value.trim().is_empty()) {
        findings.push(check_server_port(addr).await);
    } else {
        findings.push(DiagnosticFinding {
            id: "server.port".to_string(),
            label: "help.deployment.findings.serverPort.label".to_string(),
            status: "warning".to_string(),
            reason: "help.deployment.findings.serverPort.reason.missing".to_string(),
            possible_cause: "help.deployment.findings.serverPort.possibleCause.missing".to_string(),
            solution: "help.deployment.findings.serverPort.solution.missing".to_string(),
            elapsed_ms: None,
        });
    }

    let failed = findings.iter().any(|finding| finding.status == "error");
    let warning = findings.iter().any(|finding| finding.status == "warning");
    let summary = if failed {
        "help.deployment.summaryIssue"
    } else if warning {
        "help.deployment.summaryIssue"
    } else {
        "help.deployment.summaryOk"
    };

    Ok(DeploymentCheckReport {
        ok: !failed,
        checked_at,
        summary: summary.to_string(),
        findings,
    })
}

#[tauri::command]
pub async fn diagnostics_collect_system_info(
    state: State<'_, ClientRuntimeState>,
) -> CommandResult<SystemInfoReport> {
    let config = state.config().await;
    let server_version = config
        .get("server.version")
        .and_then(Value::as_str)
        .unwrap_or(DIAGNOSTIC_VALUE_DISCONNECTED)
        .to_string();
    let protocol_version = config
        .get("protocol.version")
        .and_then(Value::as_str)
        .unwrap_or("v1")
        .to_string();

    Ok(SystemInfoReport {
        client_version: env!("CARGO_PKG_VERSION").to_string(),
        server_version,
        protocol_version,
        rust_version: env!("GATE_RUSTC_VERSION").to_string(),
        os: env::consts::OS.to_string(),
        arch: env::consts::ARCH.to_string(),
        cpu: std::thread::available_parallelism()
            .map(|count| format!("{} logical cores", count.get()))
            .unwrap_or_else(|_| "unknown".to_string()),
        memory: DIAGNOSTIC_VALUE_MEMORY_PERMISSION_REQUIRED.to_string(),
        config_dir: config_path()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        log_dir: log_dir().display().to_string(),
        current_dir: env::current_dir()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
    })
}

async fn test_connection(
    server_addr: String,
    token: String,
    timeout_ms: u64,
) -> ConnectionTestReport {
    let started = Instant::now();
    let timeout_duration = Duration::from_millis(timeout_ms.max(500));

    if token.trim().is_empty() {
        return report(
            false,
            "TOKEN_EMPTY",
            "TOKEN_EMPTY",
            "TOKEN_EMPTY_REASON",
            "TOKEN_EMPTY_CAUSE",
            "TOKEN_EMPTY_SOLUTION",
            started.elapsed().as_millis(),
        );
    }

    let (host, port) = match parse_server_addr(&server_addr) {
        Ok(value) => value,
        Err(reason) => {
            return report(
                false,
                "ADDRESS_INVALID",
                "ADDRESS_INVALID",
                &reason,
                "ADDRESS_INVALID_CAUSE",
                "ADDRESS_INVALID_SOLUTION",
                started.elapsed().as_millis(),
            )
        }
    };

    let socket_addr = match timeout(timeout_duration, resolve_addr(&host, port)).await {
        Ok(Ok(addr)) => addr,
        Ok(Err(reason)) => {
            return report(
                false,
                "DNS_ERROR",
                "DNS_ERROR",
                &reason,
                "DNS_ERROR_CAUSE",
                "DNS_ERROR_SOLUTION",
                started.elapsed().as_millis(),
            )
        }
        Err(_) => {
            return report(
                false,
                "TIMEOUT",
                "TIMEOUT",
                "DNS_TIMEOUT_REASON",
                "DNS_TIMEOUT_CAUSE",
                "DNS_TIMEOUT_SOLUTION",
                started.elapsed().as_millis(),
            )
        }
    };

    match timeout(timeout_duration, TcpStream::connect(socket_addr)).await {
        Ok(Ok(stream)) => drop(stream),
        Ok(Err(error)) => {
            let code = if error.kind() == std::io::ErrorKind::ConnectionRefused {
                "SERVER_NOT_STARTED"
            } else {
                "PORT_UNREACHABLE"
            };
            let title = if code == "SERVER_NOT_STARTED" {
                "SERVER_NOT_STARTED"
            } else {
                "PORT_UNREACHABLE"
            };
            let possible_cause = if code == "SERVER_NOT_STARTED" {
                "SERVER_NOT_STARTED_CAUSE"
            } else {
                "PORT_UNREACHABLE_CAUSE"
            };
            return report(
                false,
                code,
                title,
                &error.to_string(),
                possible_cause,
                "SERVER_PORT_SOLUTION",
                started.elapsed().as_millis(),
            );
        }
        Err(_) => {
            return report(
                false,
                "TIMEOUT",
                "TIMEOUT",
                "CONNECT_TIMEOUT_REASON",
                "CONNECT_TIMEOUT_CAUSE",
                "CONNECT_TIMEOUT_SOLUTION",
                started.elapsed().as_millis(),
            )
        }
    }

    let endpoint = TransportEndpoint::Tcp {
        host: host.clone(),
        port,
    };
    let transport = TcpTransport::new();
    match timeout(timeout_duration, transport.connect(endpoint)).await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            return report(
                false,
                "SERVER_NOT_STARTED",
                "SERVER_NOT_STARTED",
                &error.to_string(),
                "SERVER_PROTOCOL_UNAVAILABLE_CAUSE",
                "SERVER_PROTOCOL_UNAVAILABLE_SOLUTION",
                started.elapsed().as_millis(),
            )
        }
        Err(_) => {
            return report(
                false,
                "TIMEOUT",
                "TIMEOUT",
                "PROTOCOL_TIMEOUT_REASON",
                "PROTOCOL_TIMEOUT_CAUSE",
                "PROTOCOL_TIMEOUT_SOLUTION",
                started.elapsed().as_millis(),
            )
        }
    }

    let request = Message::request(
        Command::AuthLogin,
        Body::Json(json!({ "token": token })),
        Metadata::default(),
    );

    match timeout(timeout_duration, transport.send(request)).await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            let _ = transport.disconnect().await;
            return report(
                false,
                "SERVER_NOT_STARTED",
                "SERVER_NOT_STARTED",
                &error.to_string(),
                "AUTH_SEND_FAILED_CAUSE",
                "AUTH_SEND_FAILED_SOLUTION",
                started.elapsed().as_millis(),
            );
        }
        Err(_) => {
            let _ = transport.disconnect().await;
            return report(
                false,
                "TIMEOUT",
                "TIMEOUT",
                "AUTH_SEND_TIMEOUT_REASON",
                "AUTH_SEND_TIMEOUT_CAUSE",
                "AUTH_SEND_TIMEOUT_SOLUTION",
                started.elapsed().as_millis(),
            );
        }
    }

    let auth_response = timeout(timeout_duration, transport.receive()).await;
    let _ = transport.disconnect().await;

    match auth_response {
        Ok(Ok(Some(message))) => match message.body {
            Body::Json(value) if value.get("ok").and_then(Value::as_bool) == Some(true) => report(
                true,
                "OK",
                "OK",
                "OK_REASON",
                "NONE",
                "OK_SOLUTION",
                started.elapsed().as_millis(),
            ),
            Body::Json(value) => report(
                false,
                "TOKEN_ERROR",
                "TOKEN_ERROR",
                &value.to_string(),
                "TOKEN_ERROR_CAUSE",
                "TOKEN_ERROR_SOLUTION",
                started.elapsed().as_millis(),
            ),
            _ => report(
                false,
                "SERVER_NOT_STARTED",
                "SERVER_NOT_STARTED",
                "AUTH_RESPONSE_INVALID_REASON",
                "AUTH_RESPONSE_INVALID_CAUSE",
                "AUTH_RESPONSE_INVALID_SOLUTION",
                started.elapsed().as_millis(),
            ),
        },
        Ok(Ok(None)) => report(
            false,
            "SERVER_NOT_STARTED",
            "SERVER_NOT_STARTED",
            "AUTH_CONNECTION_CLOSED_REASON",
            "AUTH_CONNECTION_CLOSED_CAUSE",
            "AUTH_CONNECTION_CLOSED_SOLUTION",
            started.elapsed().as_millis(),
        ),
        Ok(Err(error)) => report(
            false,
            "SERVER_NOT_STARTED",
            "SERVER_NOT_STARTED",
            &error.to_string(),
            "AUTH_RESPONSE_FAILED_CAUSE",
            "AUTH_RESPONSE_FAILED_SOLUTION",
            started.elapsed().as_millis(),
        ),
        Err(_) => report(
            false,
            "TIMEOUT",
            "TIMEOUT",
            "AUTH_TIMEOUT_REASON",
            "AUTH_TIMEOUT_CAUSE",
            "AUTH_TIMEOUT_SOLUTION",
            started.elapsed().as_millis(),
        ),
    }
}

fn report(
    ok: bool,
    code: &str,
    title: &str,
    reason: &str,
    possible_cause: &str,
    solution: &str,
    elapsed_ms: u128,
) -> ConnectionTestReport {
    ConnectionTestReport {
        ok,
        code: code.to_string(),
        title: title.to_string(),
        reason: reason.to_string(),
        possible_cause: possible_cause.to_string(),
        solution: solution.to_string(),
        elapsed_ms,
        checked_at: now_ms(),
        actions: vec![
            DiagnosticAction {
                label: "help.actions.openLogs".to_string(),
                description: "help.actions.openLogsDescription".to_string(),
            },
            DiagnosticAction {
                label: "help.actions.copyError".to_string(),
                description: "help.actions.copyErrorDescription".to_string(),
            },
        ],
    }
}

fn parse_server_addr(server_addr: &str) -> Result<(String, u16), String> {
    let trimmed = server_addr.trim();
    let (host, port) = trimmed
        .rsplit_once(':')
        .ok_or_else(|| "ADDRESS_PORT_REQUIRED".to_string())?;
    let port = port
        .parse::<u16>()
        .map_err(|_| "ADDRESS_PORT_INVALID".to_string())?;
    if host.trim().is_empty() {
        return Err("ADDRESS_HOST_EMPTY".to_string());
    }
    Ok((host.trim().to_string(), port))
}

async fn resolve_addr(host: &str, port: u16) -> Result<SocketAddr, String> {
    let mut addrs = tokio::net::lookup_host((host, port))
        .await
        .map_err(|error| error.to_string())?;
    addrs.next().ok_or_else(|| "DNS_NO_ADDRESS".to_string())
}

fn check_current_exe() -> DiagnosticFinding {
    match env::current_exe() {
        Ok(path) if path.exists() => DiagnosticFinding {
            id: "client.executable".to_string(),
            label: "help.deployment.findings.clientExecutable.label".to_string(),
            status: "ok".to_string(),
            reason: "help.deployment.findings.clientExecutable.reason.ok".to_string(),
            possible_cause: "help.deployment.findings.clientExecutable.possibleCause.ok"
                .to_string(),
            solution: "help.deployment.findings.clientExecutable.solution.ok".to_string(),
            elapsed_ms: None,
        },
        Ok(_path) => DiagnosticFinding {
            id: "client.executable".to_string(),
            label: "help.deployment.findings.clientExecutable.label".to_string(),
            status: "warning".to_string(),
            reason: "help.deployment.findings.clientExecutable.reason.missing".to_string(),
            possible_cause: "help.deployment.findings.clientExecutable.possibleCause.missing"
                .to_string(),
            solution: "help.deployment.findings.clientExecutable.solution.missing".to_string(),
            elapsed_ms: None,
        },
        Err(_error) => DiagnosticFinding {
            id: "client.executable".to_string(),
            label: "help.deployment.findings.clientExecutable.label".to_string(),
            status: "error".to_string(),
            reason: "help.deployment.findings.clientExecutable.reason.error".to_string(),
            possible_cause: "help.deployment.findings.clientExecutable.possibleCause.error"
                .to_string(),
            solution: "help.deployment.findings.clientExecutable.solution.error".to_string(),
            elapsed_ms: None,
        },
    }
}

fn check_current_dir() -> DiagnosticFinding {
    match env::current_dir() {
        Ok(_path) => DiagnosticFinding {
            id: "client.cwd".to_string(),
            label: "help.deployment.findings.clientCwd.label".to_string(),
            status: "ok".to_string(),
            reason: "help.deployment.findings.clientCwd.reason.ok".to_string(),
            possible_cause: "help.deployment.findings.clientCwd.possibleCause.ok".to_string(),
            solution: "help.deployment.findings.clientCwd.solution.ok".to_string(),
            elapsed_ms: None,
        },
        Err(_error) => DiagnosticFinding {
            id: "client.cwd".to_string(),
            label: "help.deployment.findings.clientCwd.label".to_string(),
            status: "error".to_string(),
            reason: "help.deployment.findings.clientCwd.reason.error".to_string(),
            possible_cause: "help.deployment.findings.clientCwd.possibleCause.error".to_string(),
            solution: "help.deployment.findings.clientCwd.solution.error".to_string(),
            elapsed_ms: None,
        },
    }
}

fn check_config_file() -> DiagnosticFinding {
    let path = config_path().unwrap_or_else(|| PathBuf::from("gate.toml"));
    let exists = path.exists();
    DiagnosticFinding {
        id: "config.file".to_string(),
        label: "help.deployment.findings.configFile.label".to_string(),
        status: if exists { "ok" } else { "warning" }.to_string(),
        reason: if exists {
            "help.deployment.findings.configFile.reason.ok".to_string()
        } else {
            "help.deployment.findings.configFile.reason.missing".to_string()
        },
        possible_cause: if exists {
            "help.deployment.findings.configFile.possibleCause.ok".to_string()
        } else {
            "help.deployment.findings.configFile.possibleCause.missing".to_string()
        },
        solution: if exists {
            "help.deployment.findings.configFile.solution.ok".to_string()
        } else {
            "help.deployment.findings.configFile.solution.missing".to_string()
        },
        elapsed_ms: None,
    }
}

fn check_log_dir() -> DiagnosticFinding {
    let path = log_dir();
    let status = if path.exists() { "ok" } else { "warning" };
    DiagnosticFinding {
        id: "logs.dir".to_string(),
        label: "help.deployment.findings.logsDir.label".to_string(),
        status: status.to_string(),
        reason: if path.exists() {
            "help.deployment.findings.logsDir.reason.ok".to_string()
        } else {
            "help.deployment.findings.logsDir.reason.missing".to_string()
        },
        possible_cause: if path.exists() {
            "help.deployment.findings.logsDir.possibleCause.ok".to_string()
        } else {
            "help.deployment.findings.logsDir.possibleCause.missing".to_string()
        },
        solution: if path.exists() {
            "help.deployment.findings.logsDir.solution.ok".to_string()
        } else {
            "help.deployment.findings.logsDir.solution.missing".to_string()
        },
        elapsed_ms: None,
    }
}

fn check_temp_permission() -> DiagnosticFinding {
    let path = env::temp_dir().join("gate-permission-check.tmp");
    let result = fs::write(&path, b"gate").and_then(|_| fs::remove_file(&path));
    match result {
        Ok(_) => DiagnosticFinding {
            id: "permission.write".to_string(),
            label: "help.deployment.findings.permissionWrite.label".to_string(),
            status: "ok".to_string(),
            reason: "help.deployment.findings.permissionWrite.reason.ok".to_string(),
            possible_cause: "help.deployment.findings.permissionWrite.possibleCause.ok".to_string(),
            solution: "help.deployment.findings.permissionWrite.solution.ok".to_string(),
            elapsed_ms: None,
        },
        Err(_error) => DiagnosticFinding {
            id: "permission.write".to_string(),
            label: "help.deployment.findings.permissionWrite.label".to_string(),
            status: "error".to_string(),
            reason: "help.deployment.findings.permissionWrite.reason.error".to_string(),
            possible_cause: "help.deployment.findings.permissionWrite.possibleCause.error"
                .to_string(),
            solution: "help.deployment.findings.permissionWrite.solution.error".to_string(),
            elapsed_ms: None,
        },
    }
}

async fn check_runtime_config(state: State<'_, ClientRuntimeState>) -> DiagnosticFinding {
    let config = state.config().await;
    let has_transport = config.get("network.transport").is_some();
    let has_auth = config.get("authentication.required").is_some();
    DiagnosticFinding {
        id: "config.valid".to_string(),
        label: "help.deployment.findings.configValid.label".to_string(),
        status: if has_transport && has_auth {
            "ok"
        } else {
            "warning"
        }
        .to_string(),
        reason: if has_transport && has_auth {
            "help.deployment.findings.configValid.reason.ok".to_string()
        } else {
            "help.deployment.findings.configValid.reason.missing".to_string()
        },
        possible_cause: if has_transport && has_auth {
            "help.deployment.findings.configValid.possibleCause.ok".to_string()
        } else {
            "help.deployment.findings.configValid.possibleCause.missing".to_string()
        },
        solution: if has_transport && has_auth {
            "help.deployment.findings.configValid.solution.ok".to_string()
        } else {
            "help.deployment.findings.configValid.solution.missing".to_string()
        },
        elapsed_ms: None,
    }
}

async fn check_server_port(server_addr: String) -> DiagnosticFinding {
    let started = Instant::now();
    match parse_server_addr(&server_addr) {
        Ok((host, port)) => match timeout(Duration::from_millis(2500), resolve_addr(&host, port))
            .await
        {
            Ok(Ok(addr)) => {
                match timeout(Duration::from_millis(2500), TcpStream::connect(addr)).await {
                    Ok(Ok(stream)) => {
                        drop(stream);
                        DiagnosticFinding {
                            id: "server.port".to_string(),
                            label: "help.deployment.findings.serverPort.label".to_string(),
                            status: "ok".to_string(),
                            reason: "help.deployment.findings.serverPort.reason.ok".to_string(),
                            possible_cause: "help.deployment.findings.serverPort.possibleCause.ok"
                                .to_string(),
                            solution: "help.deployment.findings.serverPort.solution.ok".to_string(),
                            elapsed_ms: Some(started.elapsed().as_millis()),
                        }
                    }
                    Ok(Err(_error)) => DiagnosticFinding {
                        id: "server.port".to_string(),
                        label: "help.deployment.findings.serverPort.label".to_string(),
                        status: "error".to_string(),
                        reason: "help.deployment.findings.serverPort.reason.connectFailed"
                            .to_string(),
                        possible_cause:
                            "help.deployment.findings.serverPort.possibleCause.connectFailed"
                                .to_string(),
                        solution: "help.deployment.findings.serverPort.solution.connectFailed"
                            .to_string(),
                        elapsed_ms: Some(started.elapsed().as_millis()),
                    },
                    Err(_) => DiagnosticFinding {
                        id: "server.port".to_string(),
                        label: "help.deployment.findings.serverPort.label".to_string(),
                        status: "error".to_string(),
                        reason: "help.deployment.findings.serverPort.reason.timeout".to_string(),
                        possible_cause: "help.deployment.findings.serverPort.possibleCause.timeout"
                            .to_string(),
                        solution: "help.deployment.findings.serverPort.solution.timeout"
                            .to_string(),
                        elapsed_ms: Some(started.elapsed().as_millis()),
                    },
                }
            }
            Ok(Err(_error)) => DiagnosticFinding {
                id: "server.port".to_string(),
                label: "help.deployment.findings.serverPort.label".to_string(),
                status: "error".to_string(),
                reason: "help.deployment.findings.serverPort.reason.dnsFailed".to_string(),
                possible_cause: "help.deployment.findings.serverPort.possibleCause.dnsFailed"
                    .to_string(),
                solution: "help.deployment.findings.serverPort.solution.dnsFailed".to_string(),
                elapsed_ms: Some(started.elapsed().as_millis()),
            },
            Err(_) => DiagnosticFinding {
                id: "server.port".to_string(),
                label: "help.deployment.findings.serverPort.label".to_string(),
                status: "error".to_string(),
                reason: "help.deployment.findings.serverPort.reason.dnsTimeout".to_string(),
                possible_cause: "help.deployment.findings.serverPort.possibleCause.dnsTimeout"
                    .to_string(),
                solution: "help.deployment.findings.serverPort.solution.dnsTimeout".to_string(),
                elapsed_ms: Some(started.elapsed().as_millis()),
            },
        },
        Err(_reason) => DiagnosticFinding {
            id: "server.port".to_string(),
            label: "help.deployment.findings.serverPort.label".to_string(),
            status: "warning".to_string(),
            reason: "help.deployment.findings.serverPort.reason.invalid".to_string(),
            possible_cause: "help.deployment.findings.serverPort.possibleCause.invalid".to_string(),
            solution: "help.deployment.findings.serverPort.solution.invalid".to_string(),
            elapsed_ms: Some(started.elapsed().as_millis()),
        },
    }
}

fn config_path() -> Option<PathBuf> {
    env::current_dir().ok().map(|path| path.join("gate.toml"))
}

fn log_dir() -> PathBuf {
    env::current_dir()
        .unwrap_or_else(|_| env::temp_dir())
        .join("logs")
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default()
}
