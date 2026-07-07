use gate_integration::{RuntimeHarness, RuntimeHarnessConfig};

#[tokio::test]
async fn forward_pipeline_transfers_bidirectional_tcp_bytes() -> anyhow::Result<()> {
    let harness = RuntimeHarness::start(RuntimeHarnessConfig::default()).await?;

    for payload in [
        b"request".as_slice(),
        b"response".as_slice(),
        b"event".as_slice(),
    ] {
        let echoed = harness.forward_once(payload).await?;
        assert_eq!(echoed, payload);
    }

    let metrics = harness.runtime().metrics();
    assert!(metrics.active_connection <= metrics.active_session);
    assert!(metrics.average_speed > 0);

    harness.shutdown().await?;
    Ok(())
}
