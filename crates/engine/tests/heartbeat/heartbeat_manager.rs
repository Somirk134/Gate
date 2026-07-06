use gate_engine::heartbeat::{HeartbeatManager, HeartbeatState};
use gate_engine::{HeartbeatConfig, TunnelId};
use std::time::Duration;

#[tokio::test]
async fn heartbeat_tracks_ping_pong_and_timeout() {
    let manager = HeartbeatManager::new(
        HeartbeatConfig::builder()
            .interval(Duration::from_millis(10))
            .timeout(Duration::from_millis(10))
            .retry_count(2)
            .retry_delay(Duration::from_millis(1))
            .max_missed_heartbeat(2)
            .build(),
    );
    let tunnel_id = TunnelId::new();

    let started = manager.start(tunnel_id).await.expect("heartbeat starts");
    assert_eq!(started.state, HeartbeatState::Running);

    let ping = manager.ping(tunnel_id).await.expect("ping recorded");
    assert_eq!(ping.state, HeartbeatState::WaitingPong);
    assert_eq!(ping.sequence, 1);

    let pong = manager
        .pong(tunnel_id, ping.sequence)
        .await
        .expect("pong recorded");
    assert_eq!(pong.state, HeartbeatState::Running);
    assert_eq!(pong.metrics.pong_count, 1);

    manager
        .ping(tunnel_id)
        .await
        .expect("second ping recorded");
    let timeout = manager.timeout(tunnel_id).await.expect("timeout recorded");
    assert_eq!(timeout.state, HeartbeatState::Timeout);
    assert_eq!(timeout.metrics.timeout_count, 1);
}

#[tokio::test]
async fn heartbeat_pause_and_resume_are_async_operations() {
    let manager = HeartbeatManager::default();
    let tunnel_id = TunnelId::new();

    manager.start(tunnel_id).await.expect("heartbeat starts");
    let paused = manager.pause(tunnel_id).await.expect("heartbeat pauses");
    assert_eq!(paused.state, HeartbeatState::Idle);

    let resumed = manager.resume(tunnel_id).await.expect("heartbeat resumes");
    assert_eq!(resumed.state, HeartbeatState::Running);
}
