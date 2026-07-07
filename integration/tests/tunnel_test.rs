use gate_engine::runtime::RuntimeState;
use gate_integration::{RuntimeHarness, RuntimeHarnessConfig};

#[tokio::test]
async fn tunnel_runtime_starts_listener_and_accepts_session() -> anyhow::Result<()> {
    let harness = RuntimeHarness::start(RuntimeHarnessConfig::default()).await?;
    assert_eq!(
        harness.runtime().context().state.current(),
        RuntimeState::Running
    );

    let echoed = harness.forward_once(b"alpha-tunnel").await?;
    assert_eq!(echoed, b"alpha-tunnel");

    let metrics = harness.runtime().metrics();
    assert!(metrics.upload >= "alpha-tunnel".len() as u64);
    assert!(metrics.download >= "alpha-tunnel".len() as u64);

    harness.shutdown().await?;
    Ok(())
}
