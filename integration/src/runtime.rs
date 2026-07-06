use anyhow::Result;
use gate_engine::runtime::{RuntimeConfig, TimeoutConfig, TunnelRuntime};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task::JoinHandle,
};

/// Configuration for a real TCP tunnel runtime test harness.
#[derive(Debug, Clone)]
pub struct RuntimeHarnessConfig {
    pub listen_addr: SocketAddr,
    pub shutdown_timeout: Duration,
}

impl Default for RuntimeHarnessConfig {
    fn default() -> Self {
        Self {
            listen_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
            shutdown_timeout: Duration::from_secs(2),
        }
    }
}

/// Owns a real `TunnelRuntime` and a local echo target service.
pub struct RuntimeHarness {
    runtime: TunnelRuntime,
    target_task: Mutex<Option<JoinHandle<()>>>,
}

impl RuntimeHarness {
    pub async fn start(config: RuntimeHarnessConfig) -> Result<Self> {
        let target_listener = TcpListener::bind("127.0.0.1:0").await?;
        let target_addr = target_listener.local_addr()?;
        let target_task = tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = target_listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let (mut reader, mut writer) = stream.into_split();
                    let _ = tokio::io::copy(&mut reader, &mut writer).await;
                });
            }
        });

        let runtime = TunnelRuntime::new(
            RuntimeConfig::builder()
                .name("alpha-v1-runtime")
                .listen_addr(config.listen_addr)
                .target_addr(target_addr)
                .timeout(
                    TimeoutConfig::builder()
                        .connect_timeout(Duration::from_secs(1))
                        .idle_timeout(Duration::from_secs(10))
                        .shutdown_timeout(config.shutdown_timeout)
                        .build(),
                )
                .monitor_interval(Duration::from_millis(100))
                .cleanup_interval(Duration::from_secs(10))
                .build(),
        );
        runtime.start().await?;
        Ok(Self {
            runtime,
            target_task: Mutex::new(Some(target_task)),
        })
    }

    pub fn runtime(&self) -> &TunnelRuntime {
        &self.runtime
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.runtime
            .bound_addr()
            .expect("runtime should expose bound listener address after start")
    }

    pub async fn forward_once(&self, payload: &[u8]) -> Result<Vec<u8>> {
        let mut client = TcpStream::connect(self.listen_addr()).await?;
        client.write_all(payload).await?;
        let mut echoed = vec![0_u8; payload.len()];
        client.read_exact(&mut echoed).await?;
        Ok(echoed)
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.runtime.shutdown().await?;
        if let Some(task) = self.target_task.lock().await.take() {
            task.abort();
        }
        Ok(())
    }
}
