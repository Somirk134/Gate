use gate_client::runtime::{ClientRuntimeState, CreateServerRequest};
use serde_json::{json, Value};
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::time;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ClientConfig::from_env()?;
    let runtime = ClientRuntimeState::default();

    if config.restore_only {
        let recovered = runtime
            .recover_after_startup()
            .await
            .map_err(anyhow::Error::msg)?;
        if recovered.is_none() {
            anyhow::bail!("没有可恢复的客户端配置");
        }
    } else {
        let server_id = runtime
            .create_server(CreateServerRequest {
                name: "e2e-server".to_string(),
                kind: Some("e2e".to_string()),
                host: config.server_host.clone(),
                port: config.server_port,
                token: config.token.clone(),
                region: Some("local".to_string()),
                remark: Some("Phase 3 e2e validation".to_string()),
                tags: Some(vec!["e2e".to_string()]),
                heartbeat_interval: Some(config.heartbeat_interval.as_secs().max(1)),
                reconnect_interval: Some(1),
                auto_connect: Some(true),
            })
            .await
            .map_err(anyhow::Error::msg)?;
        runtime
            .connect_server(server_id)
            .await
            .map_err(anyhow::Error::msg)?;
        let tunnel_id = runtime
            .create_tunnel(
                config.local_port,
                config.remote_port,
                config.protocol.clone(),
                None,
                Some(config.local_host.clone()),
                config.host.clone(),
                config.path.clone(),
            )
            .await
            .map_err(anyhow::Error::msg)?;
        runtime
            .start_tunnel(tunnel_id)
            .await
            .map_err(anyhow::Error::msg)?;
    }

    write_status(&runtime, &config, "ready").await?;

    let mut heartbeat = time::interval(config.heartbeat_interval);
    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                write_status(&runtime, &config, "stopping").await?;
                break;
            }
            _ = heartbeat.tick() => {
                let heartbeat_result = runtime.heartbeat().await;
                write_status_with_heartbeat(&runtime, &config, heartbeat_result).await?;
            }
        }
    }

    Ok(())
}

struct ClientConfig {
    server_host: String,
    server_port: u16,
    token: String,
    local_host: String,
    local_port: u16,
    remote_port: u16,
    protocol: String,
    host: Option<String>,
    path: Option<String>,
    heartbeat_interval: Duration,
    restore_only: bool,
    ready_file: Option<PathBuf>,
    status_file: Option<PathBuf>,
}

impl ClientConfig {
    fn from_env() -> anyhow::Result<Self> {
        let server_addr = required_env("GATE_E2E_SERVER_ADDR")?;
        let (server_host, server_port) = split_host_port(&server_addr)?;
        Ok(Self {
            server_host,
            server_port,
            token: env::var("GATE_AUTH_TOKEN").unwrap_or_else(|_| "gate-alpha-token".to_string()),
            local_host: env::var("GATE_E2E_LOCAL_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            local_port: required_env("GATE_E2E_LOCAL_PORT")?.parse()?,
            remote_port: required_env("GATE_E2E_REMOTE_PORT")?.parse()?,
            protocol: env::var("GATE_E2E_PROTOCOL").unwrap_or_else(|_| "tcp".to_string()),
            host: env::var("GATE_E2E_HOST")
                .ok()
                .filter(|value| !value.is_empty()),
            path: env::var("GATE_E2E_PATH")
                .ok()
                .filter(|value| !value.is_empty()),
            heartbeat_interval: Duration::from_millis(
                env::var("GATE_E2E_HEARTBEAT_MS")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(500),
            ),
            restore_only: env_bool("GATE_E2E_RESTORE_ONLY"),
            ready_file: env::var("GATE_E2E_READY_FILE").ok().map(PathBuf::from),
            status_file: env::var("GATE_E2E_STATUS_FILE").ok().map(PathBuf::from),
        })
    }
}

async fn write_status(
    runtime: &ClientRuntimeState,
    config: &ClientConfig,
    phase: &str,
) -> anyhow::Result<()> {
    write_status_with_heartbeat(runtime, config, Ok(0)).await?;
    if let Some(path) = &config.ready_file {
        let dashboard = runtime.dashboard().await;
        let statistics = runtime.statistics().await;
        write_json(
            path,
            json!({
                "phase": phase,
                "protocol": config.protocol,
                "remotePort": config.remote_port,
                "dashboard": dashboard,
                "statistics": statistics
            }),
        )?;
    }
    Ok(())
}

async fn write_status_with_heartbeat(
    runtime: &ClientRuntimeState,
    config: &ClientConfig,
    heartbeat: Result<u64, String>,
) -> anyhow::Result<()> {
    let dashboard = runtime.dashboard().await;
    let statistics = runtime.statistics().await;
    let servers = runtime.list_servers().await;
    let status = json!({
        "kind": "client",
        "protocol": config.protocol,
        "remotePort": config.remote_port,
        "heartbeat": match heartbeat {
            Ok(rtt_ms) => json!({ "ok": true, "rttMs": rtt_ms }),
            Err(error) => json!({ "ok": false, "error": error }),
        },
        "servers": servers,
        "dashboard": dashboard,
        "statistics": statistics
    });

    if let Some(path) = &config.status_file {
        write_json(path, status)?;
    }
    Ok(())
}

fn required_env(name: &str) -> anyhow::Result<String> {
    env::var(name).map_err(|_| anyhow::anyhow!("缺少环境变量 {name}"))
}

fn split_host_port(value: &str) -> anyhow::Result<(String, u16)> {
    let (host, port) = value
        .rsplit_once(':')
        .ok_or_else(|| anyhow::anyhow!("服务端地址缺少端口：{value}"))?;
    Ok((host.to_string(), port.parse()?))
}

fn env_bool(name: &str) -> bool {
    env::var(name)
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn write_json(path: &Path, value: Value) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(&value)?)?;
    Ok(())
}
