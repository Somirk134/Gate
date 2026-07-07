pub mod bootstrap;
pub mod runtime;

pub use bootstrap::ServerBootstrap;

pub async fn healthcheck() -> anyhow::Result<()> {
    let addr = std::env::var("GATE_SERVER_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:5800".to_string())
        .replace("0.0.0.0", "127.0.0.1");
    let _stream = tokio::net::TcpStream::connect(addr).await?;
    Ok(())
}
pub use gate_engine as engine;
