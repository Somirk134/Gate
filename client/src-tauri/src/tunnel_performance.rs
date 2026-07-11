use serde::Serialize;
use std::collections::BTreeMap;
use sysinfo::System;

pub const CONFIG_KEY_MODE: &str = "tunnel.performance.mode";

const MIN_RELAY_WORKERS: usize = 32;
const MAX_RELAY_WORKERS: usize = 1024;
const MIN_MAX_CONNECTIONS: u64 = 256;
const MAX_MAX_CONNECTIONS: u64 = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunnelPerformanceMode {
    Auto,
    Conservative,
    Balanced,
    High,
    Unlimited,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelPerformanceRecommendation {
    pub mode: String,
    pub effective_mode: String,
    pub relay_workers: usize,
    pub max_connections: u64,
    pub relay_worker_wait_ms: u64,
    pub cpu_cores: usize,
    pub memory_gb: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedTunnelPerformance {
    pub relay_workers: usize,
    pub max_connections: u64,
    pub relay_worker_wait_ms: u64,
}

pub fn parse_performance_mode(value: Option<&str>) -> TunnelPerformanceMode {
    match value.unwrap_or("auto").trim().to_ascii_lowercase().as_str() {
        "conservative" | "low" => TunnelPerformanceMode::Conservative,
        "balanced" | "medium" => TunnelPerformanceMode::Balanced,
        "high" => TunnelPerformanceMode::High,
        "unlimited" | "max" => TunnelPerformanceMode::Unlimited,
        _ => TunnelPerformanceMode::Auto,
    }
}

pub fn performance_mode_label(mode: TunnelPerformanceMode) -> &'static str {
    match mode {
        TunnelPerformanceMode::Auto => "auto",
        TunnelPerformanceMode::Conservative => "conservative",
        TunnelPerformanceMode::Balanced => "balanced",
        TunnelPerformanceMode::High => "high",
        TunnelPerformanceMode::Unlimited => "unlimited",
    }
}

pub fn device_capacity_snapshot() -> (usize, f64) {
    let cpu_cores = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(4)
        .max(1);
    let mut system = System::new_all();
    system.refresh_memory();
    let memory_gb = system.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    (cpu_cores, memory_gb)
}

pub fn auto_performance_mode(cpu_cores: usize, memory_gb: f64) -> TunnelPerformanceMode {
    if memory_gb < 4.0 || cpu_cores < 2 {
        TunnelPerformanceMode::Conservative
    } else if memory_gb >= 16.0 && cpu_cores >= 8 {
        TunnelPerformanceMode::High
    } else if memory_gb >= 8.0 && cpu_cores >= 4 {
        TunnelPerformanceMode::Balanced
    } else {
        TunnelPerformanceMode::Conservative
    }
}

pub fn recommend_tunnel_performance(mode: TunnelPerformanceMode) -> TunnelPerformanceRecommendation {
    let (cpu_cores, memory_gb) = device_capacity_snapshot();
    let effective_mode = if mode == TunnelPerformanceMode::Auto {
        auto_performance_mode(cpu_cores, memory_gb)
    } else {
        mode
    };

    let (relay_workers, max_connections, relay_worker_wait_ms, reason) =
        match effective_mode {
            TunnelPerformanceMode::Conservative => (
                clamp_usize(cpu_cores.saturating_mul(8), MIN_RELAY_WORKERS, 128),
                clamp_u64(cpu_cores as u64 * 32, MIN_MAX_CONNECTIONS, 512),
                15_000,
                "conservative_device_or_low_memory",
            ),
            TunnelPerformanceMode::Balanced => (
                clamp_usize(cpu_cores.saturating_mul(16), 64, 256),
                clamp_u64(cpu_cores as u64 * 64, 512, 2_048),
                30_000,
                "balanced_for_typical_workloads",
            ),
            TunnelPerformanceMode::High => (
                clamp_usize(cpu_cores.saturating_mul(32), 128, 512),
                clamp_u64(cpu_cores as u64 * 128, 1_024, MAX_MAX_CONNECTIONS),
                60_000,
                "high_throughput_multi_user",
            ),
            TunnelPerformanceMode::Unlimited => (
                MAX_RELAY_WORKERS,
                MAX_MAX_CONNECTIONS,
                120_000,
                "unlimited_server_bandwidth",
            ),
            TunnelPerformanceMode::Auto => unreachable!("auto mode should be resolved"),
        };

    TunnelPerformanceRecommendation {
        mode: performance_mode_label(mode).to_string(),
        effective_mode: performance_mode_label(effective_mode).to_string(),
        relay_workers,
        max_connections,
        relay_worker_wait_ms,
        cpu_cores,
        memory_gb,
        reason: reason.to_string(),
    }
}

pub fn resolve_tunnel_performance(
    tunnel_performance_mode: Option<&str>,
    tunnel_relay_workers: Option<u32>,
    tunnel_max_connections: Option<u64>,
    global_config: &BTreeMap<String, String>,
) -> ResolvedTunnelPerformance {
    if let Ok(value) = std::env::var("GATE_RELAY_WORKERS_PER_TUNNEL") {
        if let Ok(workers) = value.parse::<usize>() {
            if workers > 0 {
                let recommendation = recommend_tunnel_performance(TunnelPerformanceMode::High);
                return ResolvedTunnelPerformance {
                    relay_workers: workers.clamp(1, MAX_RELAY_WORKERS),
                    max_connections: tunnel_max_connections
                        .unwrap_or(recommendation.max_connections)
                        .clamp(MIN_MAX_CONNECTIONS, MAX_MAX_CONNECTIONS),
                    relay_worker_wait_ms: recommendation.relay_worker_wait_ms,
                };
            }
        }
    }

    let mode = parse_performance_mode(
        tunnel_performance_mode.or_else(|| global_config.get(CONFIG_KEY_MODE).map(String::as_str)),
    );
    let recommendation = recommend_tunnel_performance(mode);

    ResolvedTunnelPerformance {
        relay_workers: tunnel_relay_workers
            .map(|value| value as usize)
            .unwrap_or(recommendation.relay_workers)
            .clamp(1, MAX_RELAY_WORKERS),
        max_connections: tunnel_max_connections
            .unwrap_or(recommendation.max_connections)
            .clamp(MIN_MAX_CONNECTIONS, MAX_MAX_CONNECTIONS),
        relay_worker_wait_ms: recommendation.relay_worker_wait_ms,
    }
}

fn clamp_usize(value: usize, min: usize, max: usize) -> usize {
    value.clamp(min, max)
}

fn clamp_u64(value: u64, min: u64, max: u64) -> u64 {
    value.clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_mode_prefers_high_on_capable_devices() {
        let mode = auto_performance_mode(16, 32.0);
        assert_eq!(mode, TunnelPerformanceMode::High);
    }

    #[test]
    fn unlimited_mode_targets_max_pool() {
        let recommendation = recommend_tunnel_performance(TunnelPerformanceMode::Unlimited);
        assert_eq!(recommendation.relay_workers, MAX_RELAY_WORKERS);
        assert_eq!(recommendation.max_connections, MAX_MAX_CONNECTIONS);
    }

    #[test]
    fn resolve_honors_manual_tunnel_overrides() {
        let recommendation = resolve_tunnel_performance(
            Some("balanced"),
            Some(200),
            Some(1500),
            &BTreeMap::new(),
        );
        assert_eq!(recommendation.relay_workers, 200);
        assert_eq!(recommendation.max_connections, 1500);
    }
}
