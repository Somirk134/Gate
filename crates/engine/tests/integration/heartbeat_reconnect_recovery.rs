use gate_engine::heartbeat::HeartbeatState;
use gate_engine::health::HealthStatus;
use gate_engine::session::SessionId;
use gate_engine::{
    HealthManager, HeartbeatManager, ReconnectManager, RecoveryContext, SessionRecoveryManager,
    StateSyncManager, SyncTarget, TunnelId,
};
use serde_json::json;

#[tokio::test]
async fn heartbeat_timeout_can_drive_reconnect_recovery_sync_and_health() {
    let tunnel_id = TunnelId::new();

    let heartbeat = HeartbeatManager::default();
    heartbeat.start(tunnel_id).await.expect("heartbeat starts");
    heartbeat.ping(tunnel_id).await.expect("ping recorded");
    let timeout = heartbeat.timeout(tunnel_id).await.expect("timeout recorded");
    assert_eq!(timeout.state, HeartbeatState::Timeout);

    let reconnect = ReconnectManager::default();
    reconnect
        .auto_reconnect(tunnel_id, None, "heartbeat timeout")
        .await
        .expect("queue reconnect");
    let scheduled = reconnect
        .schedule_next()
        .await
        .expect("schedule reconnect")
        .expect("request exists");
    assert_eq!(scheduled.attempt, 1);
    reconnect
        .mark_succeeded(tunnel_id)
        .await
        .expect("reconnect succeeds");

    let recovery = SessionRecoveryManager::default();
    recovery
        .capture(RecoveryContext::new(tunnel_id).session_id(SessionId::new()))
        .await;
    let recovered = recovery.recover_all(tunnel_id).await.expect("recover session");
    assert!(recovered.recovered_session);

    let sync = StateSyncManager::default();
    let synced = sync
        .synchronize(
            Some(tunnel_id),
            SyncTarget::TunnelState,
            json!({ "state": "running" }),
        )
        .await
        .expect("state synchronized");
    assert_eq!(synced.version, 1);

    let health = HealthManager::default();
    let signal = health.check_heartbeat(HeartbeatState::Running).await;
    assert_eq!(signal.status, HealthStatus::Healthy);
    let report = health.report().await;
    assert_eq!(report.status, HealthStatus::Healthy);
}
