#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|arg| arg == "--healthcheck") {
        return gate_server::healthcheck().await;
    }

    gate_server::ServerBootstrap::new().boot().await?;
    Ok(())
}
