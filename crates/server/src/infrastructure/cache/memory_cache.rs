use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct MemoryCache<V> {
    store: Arc<DashMap<String, CacheEntry<V>>>,
    ttl: Duration,
}

struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

impl<V: Clone> MemoryCache<V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            store: Arc::new(DashMap::new()),
            ttl,
        }
    }

    pub fn get(&self, key: &str) -> Option<V> {
        self.store.get(key).and_then(|entry| {
            if entry.expires_at > Instant::now() {
                Some(entry.value.clone())
            } else {
                self.store.remove(key);
                None
            }
        })
    }

    pub fn set(&self, key: String, value: V) {
        let entry = CacheEntry {
            value,
            expires_at: Instant::now() + self.ttl,
        };
        self.store.insert(key, entry);
    }

    pub fn remove(&self, key: &str) {
        self.store.remove(key);
    }

    pub fn clear(&self) {
        self.store.clear();
    }
}
