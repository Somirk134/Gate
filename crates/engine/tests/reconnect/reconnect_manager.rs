use gate_engine::reconnect::{
    ExponentialBackoffStrategy, ReconnectManager, ReconnectState, ReconnectStrategy,
};
use gate_engine::{ReconnectConfig, ReconnectStrategyConfig, TunnelId};
use std::time::Duration;

#[tokio::test]
async fn reconnect_queue_schedules_and_marks_success() {
    let manager = ReconnectManager::new(
        ReconnectConfig::builder()
            .max_attempts(3)
            .strategy(ReconnectStrategyConfig::FixedInterval {
                interval: Duration::from_millis(25),
            })
            .build(),
    );
    let tunnel_id = TunnelId::new();

    let queued = manager
        .auto_reconnect(tunnel_id, None, "heartbeat timeout")
        .await
        .expect("queue reconnect");
    assert_eq!(queued.state, ReconnectState::Queued);
    assert_eq!(manager.queue_len(), 1);

    let scheduled = manager
        .schedule_next()
        .await
        .expect("scheduler runs")
        .expect("work item exists");
    assert_eq!(scheduled.attempt, 1);
    assert_eq!(scheduled.scheduled_delay_ms, 25);

    let succeeded = manager
        .mark_succeeded(tunnel_id)
        .await
        .expect("mark succeeded");
    assert_eq!(succeeded.state, ReconnectState::Succeeded);
    assert_eq!(succeeded.reconnect_count, 1);
}

#[test]
fn exponential_backoff_strategy_caps_delay() {
    let strategy = ExponentialBackoffStrategy::new(
        Duration::from_millis(100),
        Duration::from_secs(1),
        2.0,
    );

    assert_eq!(strategy.next_delay(1), Some(Duration::from_millis(100)));
    assert_eq!(strategy.next_delay(4), Some(Duration::from_millis(800)));
    assert_eq!(strategy.next_delay(8), Some(Duration::from_secs(1)));
}
