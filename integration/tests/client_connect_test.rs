use gate_integration::{AlphaClient, AlphaClientState, AlphaServer};
use std::time::Duration;

#[tokio::test]
async fn client_connects_to_real_protocol_server() -> anyhow::Result<()> {
    let server = AlphaServer::default();
    let addr = server.start().await?;

    let mut client = AlphaClient::new();
    client.connect(addr).await?;

    assert_eq!(client.state(), AlphaClientState::Connected);
    for _ in 0..10 {
        if server.statistics().await.connection_total == 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(server.statistics().await.connection_total, 1);

    client.disconnect().await?;
    server.shutdown().await?;
    Ok(())
}
