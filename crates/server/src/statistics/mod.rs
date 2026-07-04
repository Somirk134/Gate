use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Default)]
pub struct StatisticsCollector {
    pub active_connections: AtomicU64,
    pub total_clients: AtomicU64,
    pub bytes_received: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub tunnel_counts: Arc<DashMap<String, u64>>,
}

impl StatisticsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn add_bytes_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn add_bytes_sent(&self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> StatisticsSnapshot {
        StatisticsSnapshot {
            active_connections: self.active_connections.load(Ordering::Relaxed),
            total_clients: self.total_clients.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StatisticsSnapshot {
    pub active_connections: u64,
    pub total_clients: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}
