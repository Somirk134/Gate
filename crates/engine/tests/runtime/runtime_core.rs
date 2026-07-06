use gate_engine::core::TunnelId;
use gate_engine::runtime::{
    BackoffStrategy, BufferConfig, BufferPool, RetryConfig, RuntimeConfig, SessionManager,
    SessionState, TrafficStatistics,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn retry_config_supports_linear_and_exponential_backoff() {
    let linear = RetryConfig::builder()
        .initial_delay(Duration::from_millis(100))
        .max_delay(Duration::from_secs(1))
        .strategy(BackoffStrategy::Linear)
        .build();
    assert_eq!(linear.delay_for_attempt(1), Duration::from_millis(100));
    assert_eq!(linear.delay_for_attempt(3), Duration::from_millis(300));

    let exponential = RetryConfig::builder()
        .initial_delay(Duration::from_millis(100))
        .max_delay(Duration::from_secs(1))
        .strategy(BackoffStrategy::Exponential)
        .build();
    assert_eq!(
        exponential.delay_for_attempt(4),
        Duration::from_millis(800)
    );
    assert_eq!(exponential.delay_for_attempt(8), Duration::from_secs(1));
}

#[test]
fn buffer_pool_reuses_fixed_size_buffers() {
    let pool = BufferPool::new(
        BufferConfig::builder()
            .fixed_buffer_size(1024)
            .dynamic_buffer_limit(8192)
            .pool_capacity(4)
            .build(),
    );

    let buffer = pool.acquire_fixed();
    assert!(buffer.bytes().capacity() >= 1024);
    pool.release(buffer);
    assert_eq!(pool.pooled_count(), 1);
}

#[test]
fn session_manager_tracks_lifecycle_and_traffic() {
    let traffic = Arc::new(TrafficStatistics::new());
    let sessions = SessionManager::new(Arc::clone(&traffic));
    let tunnel_id = TunnelId::new();
    let remote = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 10001);
    let local = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 20001);

    let session = sessions.create(tunnel_id, Default::default(), remote, local);
    session.traffic().record_upload(128);
    session.traffic().record_download(256);

    assert_eq!(sessions.active_count(), 1);
    assert_eq!(traffic.snapshot().session_count, 1);

    let closed = sessions
        .close(&session.id, SessionState::Closed)
        .expect("session should close");
    let snapshot = closed.snapshot();

    assert_eq!(snapshot.upload_bytes, 128);
    assert_eq!(snapshot.download_bytes, 256);
    assert_eq!(snapshot.status, SessionState::Closed);
    assert_eq!(sessions.active_count(), 0);
}

#[test]
fn runtime_config_builder_sets_tcp_addresses() {
    let listen = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 18080);
    let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 28080);
    let config = RuntimeConfig::builder()
        .name("runtime-test")
        .listen_addr(listen)
        .target_addr(target)
        .max_sessions(128)
        .build();

    assert_eq!(config.name, "runtime-test");
    assert_eq!(config.listener.listen_addr, listen);
    assert_eq!(config.connector.target_addr, target);
    assert_eq!(config.max_sessions, 128);
}
