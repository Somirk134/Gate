use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    requests: Arc<DashMap<String, Vec<Instant>>>,
    max_requests: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(DashMap::new()),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut entry = self.requests.entry(key.to_string()).or_insert_with(Vec::new);

        entry.retain(|t| now.duration_since(*t) < self.window);

        if entry.len() >= self.max_requests as usize {
            false
        } else {
            entry.push(now);
            true
        }
    }
}
