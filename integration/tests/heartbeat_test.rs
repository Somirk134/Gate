use gate_integration::{AlphaClient, AlphaClientState, AlphaServer};

#[tokio::test]
async fn heartbeat_ping_pong_updates_rtt_and_statistics() -> anyhow::Result<()> {
    let server = AlphaServer::default();
    let addr = server.start().await?;

    let mut client = AlphaClient::new();
    client.connect(addr).await?;
    client.authenticate("gate-alpha-token").await?;
    client.send_heartbeat().await?;

    assert_eq!(client.state(), AlphaClientState::Running);
    assert_eq!(client.heartbeat_snapshot().pong, 1);
    assert!(client.heartbeat_snapshot().average_rtt_ms >= 0.0);
    assert!(client.heartbeat_snapshot().last_rtt_ms.is_some());

    let statistics = server.statistics().await;
    assert_eq!(statistics.heartbeat.ping, 1);
    assert_eq!(statistics.heartbeat.pong, 1);

    client.disconnect().await?;
    server.shutdown().await?;
    Ok(())
}
