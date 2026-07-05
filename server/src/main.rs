#[tokio::main]
async fn main() -> anyhow::Result<()> {
    gate_server::ServerBootstrap::new().boot().await?;
    Ok(())
}
