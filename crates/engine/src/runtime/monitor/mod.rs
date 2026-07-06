//! Runtime traffic statistics and monitor snapshots.

use crate::runtime::session::SessionManager;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Snapshot of traffic counters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrafficSnapshot {
    pub total_upload: u64,
    pub total_download: u64,
    pub current_upload: u64,
    pub current_download: u64,
    pub packet_count: u64,
    pub session_count: u64,
    pub error_count: u64,
    pub reconnect_count: u64,
}

/// Atomic traffic statistics shared by sessions and the runtime monitor.
#[derive(Debug)]
pub struct TrafficStatistics {
    total_upload: AtomicU64,
    total_download: AtomicU64,
    current_upload: AtomicU64,
    current_download: AtomicU64,
    packet_count: AtomicU64,
    session_count: AtomicU64,
    error_count: AtomicU64,
    reconnect_count: AtomicU64,
}

impl Default for TrafficStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl TrafficStatistics {
    pub fn new() -> Self {
        Self {
            total_upload: AtomicU64::new(0),
            total_download: AtomicU64::new(0),
            current_upload: AtomicU64::new(0),
            current_download: AtomicU64::new(0),
            packet_count: AtomicU64::new(0),
            session_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            reconnect_count: AtomicU64::new(0),
        }
    }

    pub fn record_upload(&self, bytes: u64) {
        if bytes == 0 {
            return;
        }
        self.total_upload.fetch_add(bytes, Ordering::Relaxed);
        self.current_upload.fetch_add(bytes, Ordering::Relaxed);
        self.packet_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_download(&self, bytes: u64) {
        if bytes == 0 {
            return;
        }
        self.total_download.fetch_add(bytes, Ordering::Relaxed);
        self.current_download.fetch_add(bytes, Ordering::Relaxed);
        self.packet_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_session(&self) {
        self.session_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_session(&self) {
        let _ = self
            .session_count
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_sub(1)
            });
    }

    pub fn increment_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_reconnect(&self) {
        self.reconnect_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> TrafficSnapshot {
        TrafficSnapshot {
            total_upload: self.total_upload.load(Ordering::Relaxed),
            total_download: self.total_download.load(Ordering::Relaxed),
            current_upload: self.current_upload.load(Ordering::Relaxed),
            current_download: self.current_download.load(Ordering::Relaxed),
            packet_count: self.packet_count.load(Ordering::Relaxed),
            session_count: self.session_count.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
            reconnect_count: self.reconnect_count.load(Ordering::Relaxed),
        }
    }

    pub fn drain_current(&self) -> (u64, u64) {
        let upload = self.current_upload.swap(0, Ordering::Relaxed);
        let download = self.current_download.swap(0, Ordering::Relaxed);
        (upload, download)
    }
}

/// Point-in-time runtime monitor metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMetrics {
    pub active_session: u64,
    pub active_connection: u64,
    pub upload: u64,
    pub download: u64,
    pub current_speed: u64,
    pub peak_speed: u64,
    pub average_speed: u64,
    pub runtime: Duration,
    pub error_count: u64,
}

#[derive(Debug)]
struct MonitorWindow {
    last_total_bytes: u64,
    last_at: Instant,
    peak_speed: u64,
}

/// Real-time monitor for runtime and traffic metrics.
#[derive(Debug)]
pub struct RuntimeMonitor {
    traffic: Arc<TrafficStatistics>,
    sessions: Arc<SessionManager>,
    started_at: Instant,
    window: Mutex<MonitorWindow>,
}

impl RuntimeMonitor {
    pub fn new(traffic: Arc<TrafficStatistics>, sessions: Arc<SessionManager>) -> Self {
        let now = Instant::now();
        Self {
            traffic,
            sessions,
            started_at: now,
            window: Mutex::new(MonitorWindow {
                last_total_bytes: 0,
                last_at: now,
                peak_speed: 0,
            }),
        }
    }

    pub fn snapshot(&self) -> RuntimeMetrics {
        let traffic = self.traffic.snapshot();
        let total_bytes = traffic.total_upload.saturating_add(traffic.total_download);
        let now = Instant::now();
        let runtime = now.saturating_duration_since(self.started_at);

        let mut window = self.window.lock();
        let elapsed_secs = now
            .saturating_duration_since(window.last_at)
            .as_secs_f64()
            .max(0.001);
        let delta = total_bytes.saturating_sub(window.last_total_bytes);
        let current_speed = (delta as f64 / elapsed_secs) as u64;
        window.peak_speed = window.peak_speed.max(current_speed);
        window.last_total_bytes = total_bytes;
        window.last_at = now;

        let average_speed = if runtime.is_zero() {
            0
        } else {
            (total_bytes as f64 / runtime.as_secs_f64().max(0.001)) as u64
        };

        RuntimeMetrics {
            active_session: self.sessions.active_count() as u64,
            active_connection: self.sessions.active_count() as u64,
            upload: traffic.total_upload,
            download: traffic.total_download,
            current_speed,
            peak_speed: window.peak_speed,
            average_speed,
            runtime,
            error_count: traffic.error_count,
        }
    }

    pub fn traffic(&self) -> Arc<TrafficStatistics> {
        Arc::clone(&self.traffic)
    }
}
