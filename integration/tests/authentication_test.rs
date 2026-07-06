use gate_integration::{AlphaClient, AlphaClientState, AlphaServer, AlphaServerConfig};

#[tokio::test]
async fn authentication_accepts_valid_token_and_rejects_invalid_token() -> anyhow::Result<()> {
    let server = AlphaServer::new(AlphaServerConfig {
        auth_token: "valid-token".to_string(),
        ..AlphaServerConfig::default()
    });
    let addr = server.start().await?;

    let mut accepted = AlphaClient::new();
    accepted.connect(addr).await?;
    let session_id = accepted.authenticate("valid-token").await?;
    assert!(!session_id.is_empty());
    assert_eq!(accepted.state(), AlphaClientState::Authenticated);

    let mut rejected = AlphaClient::new();
    rejected.connect(addr).await?;
    let result = rejected.authenticate("wrong-token").await;
    assert!(result.is_err());
    assert_eq!(rejected.state(), AlphaClientState::Failed);

    let statistics = server.statistics().await;
    assert_eq!(statistics.auth.success, 1);
    assert_eq!(statistics.auth.failure, 1);
    assert_eq!(statistics.auth.rejected_connection, 1);

    accepted.disconnect().await?;
    server.shutdown().await?;
    Ok(())
}
