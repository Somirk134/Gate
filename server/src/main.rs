#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|arg| arg == "--healthcheck") {
        return gate_server::healthcheck().await;
    }

    tracing_subscriber::fmt::init();

    gate_server::ServerBootstrap::new().boot().await?;
    Ok(())
}
