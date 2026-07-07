use crate::runtime::ClientRuntimeState;
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
) -> Result<ConnectionTestReport, String> {
    Ok(test_connection(server_addr, token, timeout_ms.unwrap_or(5000)).await)
}

#[tauri::command]
pub async fn diagnostics_run_deployment(
    state: State<'_, ClientRuntimeState>,
    server_addr: Option<String>,
) -> Result<DeploymentCheckReport, String> {
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
            label: "监听端口".to_string(),
            status: "warning".to_string(),
            reason: "尚未填写服务器地址，无法检查服务端监听端口。".to_string(),
            possible_cause: "首次启动流程还没有进入服务器配置步骤。".to_string(),
            solution: "在下一步填写服务器地址和端口后重新运行部署检查。".to_string(),
            elapsed_ms: None,
        });
    }

    let failed = findings.iter().any(|finding| finding.status == "error");
    let warning = findings.iter().any(|finding| finding.status == "warning");
    let summary = if failed {
        "部署检查发现阻塞项，请先处理错误后再创建 Tunnel。"
    } else if warning {
        "部署检查可继续，但有项目建议确认。"
    } else {
        "部署检查通过，可以继续连接服务器。"
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
) -> Result<SystemInfoReport, String> {
    let config = state.config().await;
    let server_version = config
        .get("server.version")
        .and_then(Value::as_str)
        .unwrap_or("未连接")
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
        memory: "需要系统权限时由用户反馈补充".to_string(),
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
            "Token 为空",
            "客户端没有可用于认证的 Token。",
            "首次部署时常见于复制漏掉环境变量，或还没有在服务端生成 Token。",
            "从服务端配置或部署平台复制完整 Token，再点击测试连接。",
            started.elapsed().as_millis(),
        );
    }

    let (host, port) = match parse_server_addr(&server_addr) {
        Ok(value) => value,
        Err(reason) => {
            return report(
                false,
                "ADDRESS_INVALID",
                "服务器地址格式不正确",
                &reason,
                "地址没有使用 host:port 格式，或端口不是 1-65535。",
                "按示例填写，例如 gate.example.com:7000 或 127.0.0.1:7000。",
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
                "DNS 解析失败",
                &reason,
                "域名没有解析记录、DNS 服务不可用，或当前网络无法访问该域名。",
                "确认域名拼写、DNS 记录和本机网络后重试；也可以先用服务器 IP 测试。",
                started.elapsed().as_millis(),
            )
        }
        Err(_) => {
            return report(
                false,
                "TIMEOUT",
                "DNS 解析超时",
                "在限定时间内没有完成域名解析。",
                "当前网络 DNS 响应慢，或域名解析链路不可用。",
                "切换网络、检查 DNS 设置，或临时使用服务器 IP 地址。",
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
                "服务器未启动"
            } else {
                "端口无法访问"
            };
            let possible_cause = if code == "SERVER_NOT_STARTED" {
                "目标主机可达，但 Gate Server 没有监听该端口，或服务刚刚崩溃退出。"
            } else {
                "防火墙、安全组、NAT、端口映射或服务器监听地址阻止了访问。"
            };
            return report(
                false,
                code,
                title,
                &error.to_string(),
                possible_cause,
                "确认 Rust Server 已启动，端口已监听，并放行防火墙/安全组入站规则。",
                started.elapsed().as_millis(),
            );
        }
        Err(_) => {
            return report(
                false,
                "TIMEOUT",
                "连接超时",
                "客户端在限定时间内没有连上服务器端口。",
                "服务器网络不可达、跨境链路不稳定、防火墙丢包，或端口没有对公网开放。",
                "检查服务器安全组和监听端口；如果是内网服务器，请先确认 VPN 或内网连通性。",
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
                "服务器协议无响应",
                &error.to_string(),
                "端口可以建立 TCP 连接，但对端可能不是 Gate Server，或服务端协议尚未就绪。",
                "确认该端口运行的是 Gate Rust Server，而不是其他 TCP 服务。",
                started.elapsed().as_millis(),
            )
        }
        Err(_) => {
            return report(
                false,
                "TIMEOUT",
                "协议握手超时",
                "TCP 已连通，但客户端没有在限定时间内完成 Gate 协议握手。",
                "目标端口可能不是 Gate Server，或服务端负载过高。",
                "确认该端口运行的是 Gate Rust Server，并查看服务端日志。",
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
                "服务器无法接收认证请求",
                &error.to_string(),
                "对端关闭连接、协议版本不一致，或该端口不是 Gate Server。",
                "检查服务端版本和协议版本；必要时重启服务端后再测试。",
                started.elapsed().as_millis(),
            );
        }
        Err(_) => {
            let _ = transport.disconnect().await;
            return report(
                false,
                "TIMEOUT",
                "发送认证请求超时",
                "客户端没有在限定时间内把认证请求发送到服务器。",
                "网络链路不稳定，或服务端读取连接卡住。",
                "检查网络延迟和服务端负载后重试。",
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
                "连接测试通过",
                "服务器在线，端口可达，Token 已通过认证。",
                "无。",
                "继续创建第一个 Tunnel。",
                started.elapsed().as_millis(),
            ),
            Body::Json(value) => report(
                false,
                "TOKEN_ERROR",
                "Token 错误",
                &value.to_string(),
                "Token 填写错误、服务端已经轮换 Token，或复制时包含了额外空格。",
                "重新从服务端配置复制 Token，确认没有空格或换行后重试。",
                started.elapsed().as_millis(),
            ),
            _ => report(
                false,
                "SERVER_NOT_STARTED",
                "服务器响应格式不正确",
                "服务端返回了非 JSON 认证响应。",
                "目标端口可能不是 Gate Server，或客户端/服务端协议版本不一致。",
                "确认连接的是 Gate Server 端口，并检查版本兼容性。",
                started.elapsed().as_millis(),
            ),
        },
        Ok(Ok(None)) => report(
            false,
            "SERVER_NOT_STARTED",
            "服务器提前关闭连接",
            "认证前服务端关闭了 TCP 连接。",
            "服务端进程可能刚启动失败，或协议版本不兼容。",
            "查看服务端日志，确认 Rust Server 正常运行后重试。",
            started.elapsed().as_millis(),
        ),
        Ok(Err(error)) => report(
            false,
            "SERVER_NOT_STARTED",
            "服务器认证响应失败",
            &error.to_string(),
            "服务端连接异常、协议不兼容，或服务端正在重启。",
            "查看服务端日志并确认版本一致后重试。",
            started.elapsed().as_millis(),
        ),
        Err(_) => report(
            false,
            "TIMEOUT",
            "认证超时",
            "TCP 已连通，但服务端没有在限定时间内返回认证结果。",
            "服务端负载过高、协议处理卡住，或网络延迟过大。",
            "检查服务端日志和机器负载，必要时重启 Rust Server。",
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
                label: "查看日志".to_string(),
                description: "打开 Logs 页面查看客户端最近日志。".to_string(),
            },
            DiagnosticAction {
                label: "复制错误".to_string(),
                description: "复制结构化错误信息，方便提交反馈。".to_string(),
            },
        ],
    }
}

fn parse_server_addr(server_addr: &str) -> Result<(String, u16), String> {
    let trimmed = server_addr.trim();
    let (host, port) = trimmed
        .rsplit_once(':')
        .ok_or_else(|| "服务器地址必须包含端口。".to_string())?;
    let port = port
        .parse::<u16>()
        .map_err(|_| "端口必须是 1-65535 之间的数字。".to_string())?;
    if host.trim().is_empty() {
        return Err("服务器地址不能为空。".to_string());
    }
    Ok((host.trim().to_string(), port))
}

async fn resolve_addr(host: &str, port: u16) -> Result<SocketAddr, String> {
    let mut addrs = tokio::net::lookup_host((host, port))
        .await
        .map_err(|error| error.to_string())?;
    addrs
        .next()
        .ok_or_else(|| "DNS 没有返回可连接地址。".to_string())
}

fn check_current_exe() -> DiagnosticFinding {
    match env::current_exe() {
        Ok(path) if path.exists() => DiagnosticFinding {
            id: "client.executable".to_string(),
            label: "客户端可执行文件".to_string(),
            status: "ok".to_string(),
            reason: format!("已定位客户端：{}", path.display()),
            possible_cause: "无。".to_string(),
            solution: "继续下一步。".to_string(),
            elapsed_ms: None,
        },
        Ok(path) => DiagnosticFinding {
            id: "client.executable".to_string(),
            label: "客户端可执行文件".to_string(),
            status: "warning".to_string(),
            reason: format!("当前路径不存在：{}", path.display()),
            possible_cause: "应用可能从临时目录或打包异常位置启动。".to_string(),
            solution: "重新安装 Gate Desktop，或从正式安装目录启动。".to_string(),
            elapsed_ms: None,
        },
        Err(error) => DiagnosticFinding {
            id: "client.executable".to_string(),
            label: "客户端可执行文件".to_string(),
            status: "error".to_string(),
            reason: error.to_string(),
            possible_cause: "系统限制了读取进程路径。".to_string(),
            solution: "检查系统权限后重新启动客户端。".to_string(),
            elapsed_ms: None,
        },
    }
}

fn check_current_dir() -> DiagnosticFinding {
    match env::current_dir() {
        Ok(path) => DiagnosticFinding {
            id: "client.cwd".to_string(),
            label: "工作目录".to_string(),
            status: "ok".to_string(),
            reason: format!("当前工作目录：{}", path.display()),
            possible_cause: "无。".to_string(),
            solution: "继续下一步。".to_string(),
            elapsed_ms: None,
        },
        Err(error) => DiagnosticFinding {
            id: "client.cwd".to_string(),
            label: "工作目录".to_string(),
            status: "error".to_string(),
            reason: error.to_string(),
            possible_cause: "客户端无法读取当前目录，可能是权限或安装路径问题。".to_string(),
            solution: "将应用移动到普通用户可访问目录后重试。".to_string(),
            elapsed_ms: None,
        },
    }
}

fn check_config_file() -> DiagnosticFinding {
    let path = config_path().unwrap_or_else(|| PathBuf::from("gate.toml"));
    let exists = path.exists();
    DiagnosticFinding {
        id: "config.file".to_string(),
        label: "配置文件".to_string(),
        status: if exists { "ok" } else { "warning" }.to_string(),
        reason: if exists {
            format!("已找到配置文件：{}", path.display())
        } else {
            format!("尚未找到配置文件：{}", path.display())
        },
        possible_cause: if exists {
            "无。".to_string()
        } else {
            "首次启动尚未保存配置，或配置目录被清理。".to_string()
        },
        solution: if exists {
            "继续下一步。".to_string()
        } else {
            "完成服务器配置后 Gate 会写入配置；如已配置过，请检查配置目录权限。".to_string()
        },
        elapsed_ms: None,
    }
}

fn check_log_dir() -> DiagnosticFinding {
    let path = log_dir();
    let status = if path.exists() { "ok" } else { "warning" };
    DiagnosticFinding {
        id: "logs.dir".to_string(),
        label: "日志目录".to_string(),
        status: status.to_string(),
        reason: if path.exists() {
            format!("已找到日志目录：{}", path.display())
        } else {
            format!("尚未创建日志目录：{}", path.display())
        },
        possible_cause: if path.exists() {
            "无。".to_string()
        } else {
            "首次运行或日志目录被清理。".to_string()
        },
        solution: if path.exists() {
            "继续下一步。".to_string()
        } else {
            "启动一次客户端或在反馈页面生成诊断信息后会创建日志目录。".to_string()
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
            label: "写入权限".to_string(),
            status: "ok".to_string(),
            reason: "临时目录写入测试通过。".to_string(),
            possible_cause: "无。".to_string(),
            solution: "继续下一步。".to_string(),
            elapsed_ms: None,
        },
        Err(error) => DiagnosticFinding {
            id: "permission.write".to_string(),
            label: "写入权限".to_string(),
            status: "error".to_string(),
            reason: error.to_string(),
            possible_cause: "当前用户没有写入目录权限，或安全软件拦截了文件写入。".to_string(),
            solution: "以普通用户重新安装到用户目录，或放行 Gate 的文件写入权限。".to_string(),
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
        label: "配置合法性".to_string(),
        status: if has_transport && has_auth {
            "ok"
        } else {
            "warning"
        }
        .to_string(),
        reason: if has_transport && has_auth {
            "运行时配置包含网络传输和认证设置。".to_string()
        } else {
            "运行时配置缺少部分默认字段。".to_string()
        },
        possible_cause: if has_transport && has_auth {
            "无。".to_string()
        } else {
            "配置文件被手动修改、导入配置不完整，或旧版本配置未迁移。".to_string()
        },
        solution: if has_transport && has_auth {
            "继续下一步。".to_string()
        } else {
            "在设置中恢复默认设置，再重新填写服务器连接信息。".to_string()
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
                            label: "监听端口".to_string(),
                            status: "ok".to_string(),
                            reason: format!("{server_addr} 可以建立 TCP 连接。"),
                            possible_cause: "无。".to_string(),
                            solution: "继续测试 Token 和协议版本。".to_string(),
                            elapsed_ms: Some(started.elapsed().as_millis()),
                        }
                    }
                    Ok(Err(error)) => DiagnosticFinding {
                        id: "server.port".to_string(),
                        label: "监听端口".to_string(),
                        status: "error".to_string(),
                        reason: error.to_string(),
                        possible_cause: "服务端未监听、端口未开放或防火墙拒绝连接。".to_string(),
                        solution: "启动 Rust Server，并放行服务器入站端口。".to_string(),
                        elapsed_ms: Some(started.elapsed().as_millis()),
                    },
                    Err(_) => DiagnosticFinding {
                        id: "server.port".to_string(),
                        label: "监听端口".to_string(),
                        status: "error".to_string(),
                        reason: "端口连接超时。".to_string(),
                        possible_cause: "安全组、防火墙、NAT 或网络链路阻止访问。".to_string(),
                        solution: "检查服务器网络策略和公网访问路径。".to_string(),
                        elapsed_ms: Some(started.elapsed().as_millis()),
                    },
                }
            }
            Ok(Err(error)) => DiagnosticFinding {
                id: "server.port".to_string(),
                label: "监听端口".to_string(),
                status: "error".to_string(),
                reason: error,
                possible_cause: "DNS 解析失败或域名未配置记录。".to_string(),
                solution: "修正域名解析，或改用服务器 IP 测试。".to_string(),
                elapsed_ms: Some(started.elapsed().as_millis()),
            },
            Err(_) => DiagnosticFinding {
                id: "server.port".to_string(),
                label: "监听端口".to_string(),
                status: "error".to_string(),
                reason: "DNS 解析超时。".to_string(),
                possible_cause: "当前网络 DNS 不稳定。".to_string(),
                solution: "切换网络或使用服务器 IP。".to_string(),
                elapsed_ms: Some(started.elapsed().as_millis()),
            },
        },
        Err(reason) => DiagnosticFinding {
            id: "server.port".to_string(),
            label: "监听端口".to_string(),
            status: "warning".to_string(),
            reason,
            possible_cause: "服务器地址尚未填写完整。".to_string(),
            solution: "使用 host:port 格式填写服务器地址。".to_string(),
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
