//! Communication metrics with mock latency aggregation.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MetricsSnapshot {
    pub connected_count: u64,
    pub reconnect_count: u64,
    pub failed_count: u64,
    pub send_count: u64,
    pub receive_count: u64,
    pub average_latency_ms: f64,
}

#[derive(Debug, Default)]
pub struct CommunicationMetrics {
    connected_count: AtomicU64,
    reconnect_count: AtomicU64,
    failed_count: AtomicU64,
    send_count: AtomicU64,
    receive_count: AtomicU64,
    total_latency_ms: AtomicU64,
    latency_samples: AtomicU64,
}

impl CommunicationMetrics {
    pub fn record_connected(&self) {
        self.connected_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_reconnect(&self) {
        self.reconnect_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_failed(&self) {
        self.failed_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_send(&self) {
        self.send_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_receive(&self) {
        self.receive_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_latency(&self, latency_ms: u64) {
        self.total_latency_ms
            .fetch_add(latency_ms, Ordering::Relaxed);
        self.latency_samples.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let samples = self.latency_samples.load(Ordering::Relaxed);
        let total_latency = self.total_latency_ms.load(Ordering::Relaxed);

        MetricsSnapshot {
            connected_count: self.connected_count.load(Ordering::Relaxed),
            reconnect_count: self.reconnect_count.load(Ordering::Relaxed),
            failed_count: self.failed_count.load(Ordering::Relaxed),
            send_count: self.send_count.load(Ordering::Relaxed),
            receive_count: self.receive_count.load(Ordering::Relaxed),
            average_latency_ms: if samples == 0 {
                0.0
            } else {
                total_latency as f64 / samples as f64
            },
        }
    }
}
