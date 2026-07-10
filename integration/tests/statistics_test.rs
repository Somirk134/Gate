use gate_integration::{AlphaClient, AlphaServer};
use gate_protocol::Command;
use serde_json::{json, Value};

#[tokio::test]
async fn statistics_query_returns_runtime_counters_from_real_flow() -> anyhow::Result<()> {
    let server = AlphaServer::default();
    let addr = server.start().await?;

    let mut client = AlphaClient::new();
    client.connect(addr).await?;
    client
        .authenticate("gate-integration-test-token-20260710-release-audit")
        .await?;
    client.send_heartbeat().await?;
    let response = client.request(Command::StatisticsQuery, json!({})).await?;

    let body = gate_integration::protocol::json_body(&response)?;
    assert_eq!(body.get("ok").and_then(Value::as_bool), Some(true));
    assert_eq!(
        body.pointer("/data/auth/success").and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        body.pointer("/data/heartbeat/ping").and_then(Value::as_u64),
        Some(1)
    );
    assert!(
        body.pointer("/data/request_total")
            .and_then(Value::as_u64)
            .unwrap_or_default()
            >= 3
    );

    client.disconnect().await?;
    server.shutdown().await?;
    Ok(())
}
