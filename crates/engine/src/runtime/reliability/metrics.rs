//! Runtime reliability metrics registry.

use crate::runtime::reliability::RuntimeId;
use crate::runtime::state::RuntimeState;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unified runtime reliability metrics used by supervisors, watchdogs, and dashboards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeReliabilityMetrics {
    pub runtime_count: u64,
    pub connection_count: u64,
    pub tls_session_count: u64,
    pub memory_bytes: u64,
    pub cpu_usage_percent: f64,
    pub restart_count: u64,
    pub health_score: u8,
    pub task_count: u64,
    pub failed_task_count: u64,
    pub channel_depth: u64,
    pub runtime_states: HashMap<RuntimeState, u64>,
    pub sampled_at_millis: u64,
}

impl Default for RuntimeReliabilityMetrics {
    fn default() -> Self {
        Self {
            runtime_count: 0,
            connection_count: 0,
            tls_session_count: 0,
            memory_bytes: 0,
            cpu_usage_percent: 0.0,
            restart_count: 0,
            health_score: 100,
            task_count: 0,
            failed_task_count: 0,
            channel_depth: 0,
            runtime_states: HashMap::new(),
            sampled_at_millis: now_millis(),
        }
    }
}

impl RuntimeReliabilityMetrics {
    pub fn with_state(mut self, state: RuntimeState) -> Self {
        *self.runtime_states.entry(state).or_insert(0) += 1;
        self.runtime_count = self.runtime_count.saturating_add(1);
        self
    }
}

/// In-memory metrics registry keyed by runtime id.
#[derive(Debug, Default)]
pub struct RuntimeMetricsRegistry {
    metrics: DashMap<RuntimeId, RuntimeReliabilityMetrics>,
}

impl RuntimeMetricsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(
        &self,
        runtime_id: RuntimeId,
        metrics: RuntimeReliabilityMetrics,
    ) -> RuntimeReliabilityMetrics {
        self.metrics.insert(runtime_id, metrics.clone());
        metrics
    }

    pub fn remove(&self, runtime_id: RuntimeId) -> Option<RuntimeReliabilityMetrics> {
        self.metrics.remove(&runtime_id).map(|(_, metrics)| metrics)
    }

    pub fn get(&self, runtime_id: RuntimeId) -> Option<RuntimeReliabilityMetrics> {
        self.metrics
            .get(&runtime_id)
            .map(|entry| entry.value().clone())
    }

    pub fn aggregate(&self) -> RuntimeReliabilityMetrics {
        let mut aggregate = RuntimeReliabilityMetrics::default();
        aggregate.runtime_count = 0;
        aggregate.health_score = 0;

        let mut health_total = 0_u64;
        for entry in self.metrics.iter() {
            let metrics = entry.value();
            aggregate.runtime_count = aggregate
                .runtime_count
                .saturating_add(metrics.runtime_count.max(1));
            aggregate.connection_count = aggregate
                .connection_count
                .saturating_add(metrics.connection_count);
            aggregate.tls_session_count = aggregate
                .tls_session_count
                .saturating_add(metrics.tls_session_count);
            aggregate.memory_bytes = aggregate.memory_bytes.saturating_add(metrics.memory_bytes);
            aggregate.cpu_usage_percent += metrics.cpu_usage_percent;
            aggregate.restart_count = aggregate
                .restart_count
                .saturating_add(metrics.restart_count);
            aggregate.task_count = aggregate.task_count.saturating_add(metrics.task_count);
            aggregate.failed_task_count = aggregate
                .failed_task_count
                .saturating_add(metrics.failed_task_count);
            aggregate.channel_depth = aggregate
                .channel_depth
                .saturating_add(metrics.channel_depth);
            health_total = health_total.saturating_add(u64::from(metrics.health_score));
            for (state, count) in &metrics.runtime_states {
                *aggregate.runtime_states.entry(*state).or_insert(0) += count;
            }
        }

        aggregate.health_score = if self.metrics.is_empty() {
            100
        } else {
            (health_total / self.metrics.len() as u64) as u8
        };
        aggregate.sampled_at_millis = now_millis();
        aggregate
    }

    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.metrics.is_empty()
    }
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
