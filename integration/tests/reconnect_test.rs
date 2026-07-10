use gate_integration::{AlphaClient, AlphaClientState, AlphaServer};

#[tokio::test]
async fn client_can_reconnect_and_reauthenticate_after_disconnect() -> anyhow::Result<()> {
    let server = AlphaServer::default();
    let addr = server.start().await?;

    let mut client = AlphaClient::new();
    client.connect(addr).await?;
    client
        .authenticate("gate-integration-test-token-20260710-release-audit")
        .await?;
    client.disconnect().await?;
    assert_eq!(client.state(), AlphaClientState::Disconnected);

    client.reconnect().await?;
    assert_eq!(client.state(), AlphaClientState::Connected);
    client
        .authenticate("gate-integration-test-token-20260710-release-audit")
        .await?;
    assert_eq!(client.state(), AlphaClientState::Authenticated);

    let statistics = server.statistics().await;
    assert_eq!(statistics.auth.success, 2);
    assert!(statistics.connection_total >= 2);

    client.disconnect().await?;
    server.shutdown().await?;
    Ok(())
}
