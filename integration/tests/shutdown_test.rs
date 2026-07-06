use gate_integration::{AlphaClient, AlphaServer, AlphaServerState, RuntimeHarness, RuntimeHarnessConfig};
use gate_engine::runtime::RuntimeState;
use gate_protocol::Command;
use serde_json::json;

#[tokio::test]
async fn server_and_runtime_shutdown_gracefully() -> anyhow::Result<()> {
    let server = AlphaServer::default();
    let addr = server.start().await?;
    let harness = RuntimeHarness::start(RuntimeHarnessConfig::default()).await?;

    let mut client = AlphaClient::new();
    client.connect(addr).await?;
    client.authenticate("gate-alpha-token").await?;
    let response = client.request(Command::SystemShutdown, json!({})).await?;
    assert_eq!(response.header.command, Command::SystemShutdown);
    client.disconnect().await?;

    server.shutdown().await?;
    assert_eq!(server.state().await, AlphaServerState::Shutdown);

    harness.shutdown().await?;
    assert_eq!(harness.runtime().context().state.current(), RuntimeState::Closed);
    Ok(())
}
