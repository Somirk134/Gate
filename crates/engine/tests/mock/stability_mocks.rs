use gate_engine::connection::ConnectionId;
use gate_engine::heartbeat::HeartbeatState;
use gate_engine::mock::{MockHeartbeat, MockReconnect, MockRecovery};
use gate_engine::reconnect::ReconnectState;
use gate_engine::TunnelId;

#[tokio::test]
async fn mocks_cover_timeout_failures_and_recovery() {
    let tunnel_id = TunnelId::new();
    let connection_id = ConnectionId::new();

    let heartbeat = MockHeartbeat::default();
    let timeout = heartbeat
        .simulate_timeout(tunnel_id)
        .await
        .expect("timeout simulated");
    assert_eq!(timeout.state, HeartbeatState::Timeout);

    let reconnect = MockReconnect::default();
    let failed = reconnect
        .simulate_consecutive_failures(tunnel_id, connection_id, 2)
        .await
        .expect("failures simulated");
    assert_eq!(failed.state, ReconnectState::Failed);
    assert_eq!(failed.failed_count, 2);

    let recovery = MockRecovery::default();
    let recovered = recovery
        .simulate_recovered(tunnel_id)
        .await
        .expect("recovery simulated");
    assert!(recovered.recovered_session);
    assert!(recovered.recovered_tunnel);
}
