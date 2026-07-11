use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
use tempfile::TempDir;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::sleep,
};

const SERVER_BIN: &str = env!("CARGO_BIN_EXE_gate-e2e-server");
const CLIENT_BIN: &str = env!("CARGO_BIN_EXE_gate-e2e-client");
const LOCAL_SERVICE_BIN: &str = env!("CARGO_BIN_EXE_gate-e2e-local-service");
const AUTH_TOKEN: &str = "gate-integration-test-token-20260710-release-audit";

// These real-world e2e tests each spawn multiple real processes (server + client +
// local service). Running them in parallel causes resource contention that pushes the
// client-ready handshake past its timeout, so we serialize them behind a global lock to
// keep the suite deterministic on CI runners.
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn normal_tcp_and_http_tunnels_relay_through_real_processes() -> Result<()> {
    let _serial = SERIAL.lock().await;
    let tcp = E2eStack::start("tcp", 4).await?;
    assert_eq!(
        wait_for_tcp_roundtrip(tcp.remote_port, b"gate-tcp", Duration::from_secs(10)).await?,
        b"gate-tcp"
    );

    let http = E2eStack::start("http", 4).await?;
    let response = wait_for_http_roundtrip(http.remote_port, Duration::from_secs(10)).await?;
    assert!(response.contains("200 OK"));
    assert!(response.contains("gate-e2e-http"));
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn server_restart_recovery_restores_session_tunnel_and_public_access() -> Result<()> {
    let _serial = SERIAL.lock().await;
    let mut stack = E2eStack::start("tcp", 8).await?;
    wait_for_tcp_roundtrip(stack.remote_port, b"before", Duration::from_secs(10)).await?;

    stack.server.kill();
    sleep(Duration::from_secs(2)).await;
    stack.server = stack.spawn_server().await?;

    let started = Instant::now();
    let response =
        wait_for_tcp_roundtrip(stack.remote_port, b"after", Duration::from_secs(20)).await?;
    let recovery_time = started.elapsed();

    assert_eq!(response, b"after");
    let status =
        wait_for_status_condition(&stack.client_status, Duration::from_secs(10), |value| {
            value
                .pointer("/statistics/connection/reconnect")
                .and_then(Value::as_u64)
                .unwrap_or_default()
                >= 1
        })
        .await?;
    assert!(
        status
            .pointer("/statistics/connection/reconnect")
            .and_then(Value::as_u64)
            .unwrap_or_default()
            >= 1
    );
    assert!(
        status
            .pointer("/statistics/connection/recoveryTimeMs")
            .and_then(Value::as_u64)
            .unwrap_or_default()
            > 0
    );
    write_observation(
        &stack.root,
        "server-restart",
        json_observation(
            "server_restart_recovery_ms",
            recovery_time.as_millis() as u64,
        ),
    )?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn client_restart_recovery_restores_tunnel_without_recreating_config() -> Result<()> {
    let _serial = SERIAL.lock().await;
    let mut stack = E2eStack::start("tcp", 8).await?;
    wait_for_tcp_roundtrip(stack.remote_port, b"before", Duration::from_secs(10)).await?;

    let original_ready = wait_for_status_value(&stack.client_ready, Duration::from_secs(5)).await?;
    let original_tunnel_count = original_ready
        .pointer("/dashboard/overview/tunnelCount")
        .and_then(Value::as_u64)
        .unwrap_or_default();

    stack.client.kill();
    sleep(Duration::from_secs(1)).await;
    stack.client = stack.spawn_client(true, 8).await?;

    let response =
        wait_for_tcp_roundtrip(stack.remote_port, b"restored", Duration::from_secs(20)).await?;
    assert_eq!(response, b"restored");

    let restored = wait_for_status_value(&stack.client_ready, Duration::from_secs(5)).await?;
    assert_eq!(
        restored
            .pointer("/dashboard/overview/tunnelCount")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        original_tunnel_count
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn network_interruption_reconnects_after_short_outage() -> Result<()> {
    let _serial = SERIAL.lock().await;
    run_network_interruption(Duration::from_secs(2)).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[ignore = "full real-world matrix: 10s, 30s, 60s interruption"]
async fn network_interruption_matrix_10_30_60_seconds() -> Result<()> {
    for duration in [
        Duration::from_secs(10),
        Duration::from_secs(30),
        Duration::from_secs(60),
    ] {
        run_network_interruption(duration).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[ignore = "full stress matrix: 100, 500, 1000 concurrent public connections"]
async fn stress_matrix_100_500_1000_connections() -> Result<()> {
    for connections in [100_usize, 500, 1000] {
        run_stress(connections).await?;
    }
    Ok(())
}

async fn run_network_interruption(duration: Duration) -> Result<()> {
    let mut stack = E2eStack::start("tcp", 8).await?;
    wait_for_tcp_roundtrip(stack.remote_port, b"before", Duration::from_secs(10)).await?;

    stack.server.kill();
    sleep(duration).await;
    stack.server = stack.spawn_server().await?;

    let started = Instant::now();
    let response =
        wait_for_tcp_roundtrip(stack.remote_port, b"network", Duration::from_secs(30)).await?;
    assert_eq!(response, b"network");

    write_observation(
        &stack.root,
        "network-interruption",
        json_observation(
            &format!("network_interruption_{}_ms", duration.as_secs()),
            started.elapsed().as_millis() as u64,
        ),
    )?;
    Ok(())
}

async fn run_stress(connections: usize) -> Result<()> {
    let worker_count = connections.saturating_add(32);
    let stack = E2eStack::start("tcp", worker_count).await?;
    wait_for_tcp_roundtrip(stack.remote_port, b"warmup", Duration::from_secs(10)).await?;

    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    let memory_before = system.used_memory();
    let started = Instant::now();

    let mut tasks = Vec::with_capacity(connections);
    for index in 0..connections {
        let port = stack.remote_port;
        tasks.push(tokio::spawn(async move {
            let payload = format!("stress-{index}");
            let echoed = tcp_roundtrip(port, payload.as_bytes()).await?;
            if echoed != payload.as_bytes() {
                return Err(anyhow!("stress response mismatch"));
            }
            Ok::<(), anyhow::Error>(())
        }));
    }

    let mut errors = 0_u64;
    for task in tasks {
        if task.await?.is_err() {
            errors += 1;
        }
    }

    system.refresh_all();
    write_observation(
        &stack.root,
        "stress",
        serde_json::json!({
            "connections": connections,
            "errors": errors,
            "latencyMs": started.elapsed().as_millis() as u64,
            "memoryBeforeBytes": memory_before,
            "memoryAfterBytes": system.used_memory()
        }),
    )?;

    assert_eq!(errors, 0);
    Ok(())
}

struct E2eStack {
    root: PathBuf,
    _temp: TempDir,
    server_port: u16,
    local_port: u16,
    remote_port: u16,
    client_ready: PathBuf,
    client_status: PathBuf,
    appdata: PathBuf,
    server: ProcessGuard,
    local_service: ProcessGuard,
    client: ProcessGuard,
    protocol: String,
}

impl E2eStack {
    async fn start(protocol: &str, relay_workers: usize) -> Result<Self> {
        let temp = tempfile::tempdir()?;
        let root = temp.path().to_path_buf();
        let appdata = root.join("appdata");
        let server_port = unused_port().await?;
        let local_port = unused_port().await?;
        let remote_port = unused_port().await?;
        let client_ready = root.join("client-ready.json");
        let client_status = root.join("client-status.json");

        let mut stack = Self {
            root,
            _temp: temp,
            server_port,
            local_port,
            remote_port,
            client_ready,
            client_status,
            appdata,
            server: ProcessGuard::empty(),
            local_service: ProcessGuard::empty(),
            client: ProcessGuard::empty(),
            protocol: protocol.to_string(),
        };

        stack.server = stack.spawn_server().await?;
        stack.local_service = stack.spawn_local_service().await?;
        stack.client = stack.spawn_client(false, relay_workers).await?;
        Ok(stack)
    }

    async fn spawn_server(&self) -> Result<ProcessGuard> {
        let mut command = base_command(SERVER_BIN);
        command
            .env(
                "GATE_SERVER_ADDR",
                format!("127.0.0.1:{}", self.server_port),
            )
            .env("GATE_AUTH_TOKEN", AUTH_TOKEN)
            .env("GATE_TUNNEL_BIND_ADDR", "127.0.0.1")
            .env("GATE_HEARTBEAT_TIMEOUT_MS", "1500")
            // 服务端域名绑定也必须隔离，避免固定 e2e.local 污染用户默认数据目录。
            .env(
                "GATE_SERVER_DOMAIN_DB",
                self.root.join("server-domains.sqlite3"),
            );

        let process = ProcessGuard::spawn(command)?;
        wait_for_tcp_port(self.server_port, Duration::from_secs(5)).await?;
        Ok(process)
    }

    async fn spawn_local_service(&self) -> Result<ProcessGuard> {
        let ready = self.root.join("local-ready.json");
        let mut command = base_command(LOCAL_SERVICE_BIN);
        command
            .env("GATE_E2E_LOCAL_MODE", &self.protocol)
            .env(
                "GATE_E2E_LOCAL_ADDR",
                format!("127.0.0.1:{}", self.local_port),
            )
            .env("GATE_E2E_READY_FILE", &ready);

        let process = ProcessGuard::spawn(command)?;
        wait_for_file(&ready, Duration::from_secs(5)).await?;
        wait_for_tcp_port(self.local_port, Duration::from_secs(5)).await?;
        Ok(process)
    }

    async fn spawn_client(&self, restore_only: bool, relay_workers: usize) -> Result<ProcessGuard> {
        let _ = fs::remove_file(&self.client_ready);
        let _ = fs::remove_file(&self.client_status);
        let mut command = base_command(CLIENT_BIN);
        command
            .env(
                "GATE_E2E_SERVER_ADDR",
                format!("127.0.0.1:{}", self.server_port),
            )
            .env("GATE_AUTH_TOKEN", AUTH_TOKEN)
            .env("GATE_E2E_LOCAL_HOST", "127.0.0.1")
            .env("GATE_E2E_LOCAL_PORT", self.local_port.to_string())
            .env("GATE_E2E_REMOTE_PORT", self.remote_port.to_string())
            .env("GATE_E2E_PROTOCOL", &self.protocol)
            .env("GATE_E2E_HEARTBEAT_MS", "200")
            .env("GATE_E2E_READY_FILE", &self.client_ready)
            .env("GATE_E2E_STATUS_FILE", &self.client_status)
            .env("GATE_RECONNECT_DELAYS_MS", "100,200,500,1000")
            .env("GATE_RELAY_WORKERS_PER_TUNNEL", relay_workers.to_string())
            .env("APPDATA", &self.appdata)
            .env("XDG_DATA_HOME", &self.appdata)
            .env("HOME", &self.appdata)
            .env("GATE_DOMAIN_DB", self.root.join("domains.sqlite3"));
        if restore_only {
            command.env("GATE_E2E_RESTORE_ONLY", "1");
        }
        if self.protocol == "http" {
            command.env("GATE_E2E_HOST", "e2e.local");
            command.env("GATE_E2E_PATH", "/");
        }

        let process = ProcessGuard::spawn(command)?;
        wait_for_file(&self.client_ready, Duration::from_secs(30)).await?;
        Ok(process)
    }
}

impl Drop for E2eStack {
    fn drop(&mut self) {
        self.client.kill();
        self.local_service.kill();
        self.server.kill();
    }
}

struct ProcessGuard {
    child: Option<Child>,
}

impl ProcessGuard {
    fn empty() -> Self {
        Self { child: None }
    }

    fn spawn(mut command: Command) -> Result<Self> {
        let child = command.spawn().context("spawn e2e process")?;
        Ok(Self { child: Some(child) })
    }

    fn kill(&mut self) {
        if let Some(child) = &mut self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.child = None;
    }
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        self.kill();
    }
}

fn base_command(binary: &str) -> Command {
    let mut command = Command::new(binary);
    command.stdout(Stdio::null()).stderr(Stdio::null());
    command
}

async fn unused_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

async fn wait_for_tcp_port(port: u16, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(anyhow!("port {port} did not open"));
        }
        sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_for_file(path: &Path, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if path.exists() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(anyhow!("file did not appear: {}", path.display()));
        }
        sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_for_status_value(path: &Path, timeout: Duration) -> Result<Value> {
    wait_for_file(path, timeout).await?;
    let deadline = Instant::now() + timeout;
    loop {
        if let Ok(bytes) = fs::read(path) {
            if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
                return Ok(value);
            }
        }
        if Instant::now() >= deadline {
            return Err(anyhow!("status json was not readable: {}", path.display()));
        }
        sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_for_status_condition<F>(path: &Path, timeout: Duration, predicate: F) -> Result<Value>
where
    F: Fn(&Value) -> bool,
{
    let deadline = Instant::now() + timeout;
    loop {
        if let Ok(value) = wait_for_status_value(path, Duration::from_millis(200)).await {
            if predicate(&value) {
                return Ok(value);
            }
        }
        if Instant::now() >= deadline {
            return Err(anyhow!("status condition was not met: {}", path.display()));
        }
        sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_tcp_roundtrip(port: u16, payload: &[u8], timeout: Duration) -> Result<Vec<u8>> {
    let deadline = Instant::now() + timeout;
    loop {
        match tcp_roundtrip(port, payload).await {
            Ok(response) => return Ok(response),
            Err(error) if Instant::now() >= deadline => return Err(error),
            Err(_) => sleep(Duration::from_millis(100)).await,
        }
    }
}

async fn tcp_roundtrip(port: u16, payload: &[u8]) -> Result<Vec<u8>> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await?;
    stream.write_all(payload).await?;
    stream.shutdown().await?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await?;
    Ok(response)
}

async fn wait_for_http_roundtrip(port: u16, timeout: Duration) -> Result<String> {
    let deadline = Instant::now() + timeout;
    loop {
        match http_roundtrip(port).await {
            Ok(response) => return Ok(response),
            Err(error) if Instant::now() >= deadline => return Err(error),
            Err(_) => sleep(Duration::from_millis(100)).await,
        }
    }
}

async fn http_roundtrip(port: u16) -> Result<String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await?;
    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: e2e.local\r\nConnection: close\r\n\r\n")
        .await?;
    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    Ok(response)
}

fn write_observation(root: &Path, name: &str, value: Value) -> Result<()> {
    let path = root.join(format!("{name}.json"));
    fs::write(path, serde_json::to_vec_pretty(&value)?)?;
    Ok(())
}

fn json_observation(name: &str, value: u64) -> Value {
    serde_json::json!({
        "metric": name,
        "value": value
    })
}
