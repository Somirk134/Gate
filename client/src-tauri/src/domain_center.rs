use chrono::Utc;
use gate_server_domain::{
    model::{DnsStatus, Domain as ManagedDomain, DomainId, Host as ManagedHost, TunnelId},
    repository::{DomainRepository, SqliteRepository as SqliteDomainRepository},
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    fs,
    time::Duration,
};
use tokio::net::lookup_host;

use crate::project::ProjectWorkspaceState;
use crate::runtime::{certificate_store_root, domain_store_path, ClientRuntimeState, UpdateTunnelRequest};

const DAY_MS: i64 = 86_400_000;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainListQuery {
    pub keyword: Option<String>,
    pub health: Option<String>,
    pub protocol: Option<String>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainCreateRequest {
    pub host: String,
    #[serde(default)]
    pub tunnel_id: String,
    pub protocol: Option<String>,
    pub path: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainBindRequest {
    pub host: String,
    pub tunnel_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainBatchRequest {
    pub hosts: Vec<String>,
    pub action: String,
    pub tunnel_id: Option<String>,
    pub certificate_domain: Option<String>,
}

pub async fn domain_list(
    state: &ClientRuntimeState,
    projects: &ProjectWorkspaceState,
    query: DomainListQuery,
) -> Result<Value, String> {
    let items = collect_domain_items(state, projects).await?;
    let filtered = filter_domains(items, &query);
    let sorted = sort_domains(filtered, query.sort_by.as_deref(), query.sort_dir.as_deref());
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).clamp(1, 200);
    let total = sorted.len();
    let start = ((page - 1) as usize).saturating_mul(page_size as usize);
    let page_items = sorted
        .into_iter()
        .skip(start)
        .take(page_size as usize)
        .collect::<Vec<_>>();

    Ok(json!({
        "items": page_items,
        "total": total,
        "page": page,
        "pageSize": page_size,
        "generatedAt": Utc::now().timestamp_millis(),
        "dbPath": domain_store_path(),
    }))
}

pub async fn domain_stats(
    state: &ClientRuntimeState,
    projects: &ProjectWorkspaceState,
) -> Result<Value, String> {
    let items = collect_domain_items(state, projects).await?;
    let total = items.len() as u64;
    let online = items
        .iter()
        .filter(|item| item.get("healthStatus").and_then(Value::as_str) == Some("healthy"))
        .count() as u64;
    let https_count = items
        .iter()
        .filter(|item| item.get("https").and_then(Value::as_bool) == Some(true))
        .count() as u64;
    let http_count = total.saturating_sub(https_count);
    let expiring_soon = items
        .iter()
        .filter(|item| {
            item.get("certificate")
                .and_then(|cert| cert.get("daysRemaining"))
                .and_then(Value::as_i64)
                .map(|days| days > 0 && days <= 30)
                == Some(true)
        })
        .count() as u64;
    let abnormal = items
        .iter()
        .filter(|item| {
            !matches!(
                item.get("healthStatus").and_then(Value::as_str),
                Some("healthy")
            )
        })
        .count() as u64;
    let unbound_tunnel = items
        .iter()
        .filter(|item| item.get("tunnelId").and_then(Value::as_str).unwrap_or("").is_empty())
        .count() as u64;
    let dns_failed = items
        .iter()
        .filter(|item| {
            matches!(
                item.get("dnsStatus").and_then(Value::as_str),
                Some("mismatched") | Some("noRecord") | Some("error")
            )
        })
        .count() as u64;
    let requests_24h = items
        .iter()
        .filter_map(|item| item.get("requestCount24h").and_then(Value::as_u64))
        .sum::<u64>();
    let traffic_24h = items
        .iter()
        .filter_map(|item| item.get("traffic24h").and_then(Value::as_u64))
        .sum::<u64>();
    let request_trend = aggregate_trend(&items, "requestTrend");
    let traffic_trend = aggregate_trend(&items, "trafficTrend");

    Ok(json!({
        "total": total,
        "online": online,
        "https": https_count,
        "http": http_count,
        "expiringSoon": expiring_soon,
        "abnormal": abnormal,
        "unboundTunnel": unbound_tunnel,
        "dnsFailed": dns_failed,
        "requests24h": requests_24h,
        "traffic24h": traffic_24h,
        "requestTrend": request_trend,
        "trafficTrend": traffic_trend,
        "generatedAt": Utc::now().timestamp_millis(),
    }))
}

pub async fn domain_detail(
    state: &ClientRuntimeState,
    projects: &ProjectWorkspaceState,
    host: String,
) -> Result<Value, String> {
    let normalized = normalize_host(&host)?;
    let items = collect_domain_items(state, projects).await?;
    let summary = items
        .into_iter()
        .find(|item| item.get("host").and_then(Value::as_str) == Some(normalized.as_str()))
        .ok_or_else(|| "DOMAIN_NOT_FOUND".to_string())?;

    let dashboard = state.dashboard().await;
    let tunnels = dashboard
        .get("tunnels")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let tunnel = tunnels.iter().find(|tunnel| {
        tunnel.get("host").and_then(Value::as_str) == Some(normalized.as_str())
            || tunnel.get("id").and_then(Value::as_str)
                == summary.get("tunnelId").and_then(Value::as_str)
    });

    let servers = server_address_map(&dashboard);
    let dns = resolve_dns_snapshot(&normalized, &servers).await;
    let runtime = build_runtime_detail(tunnel, &normalized);
    let logs = filter_domain_logs(&state.logs().await, summary.get("tunnelId").and_then(Value::as_str), &normalized);

    Ok(json!({
        "summary": summary,
        "managed": load_managed_domain(&normalized),
        "dns": dns,
        "runtime": runtime,
        "logs": logs,
        "generatedAt": Utc::now().timestamp_millis(),
    }))
}

pub async fn domain_check_dns(state: &ClientRuntimeState, host: String) -> Result<Value, String> {
    let normalized = normalize_host(&host)?;
    let dashboard = state.dashboard().await;
    let servers = server_address_map(&dashboard);
    Ok(resolve_dns_snapshot(&normalized, &servers).await)
}

pub async fn domain_create(
    state: &ClientRuntimeState,
    projects: &ProjectWorkspaceState,
    request: DomainCreateRequest,
) -> Result<Value, String> {
    let host = normalize_host(&request.host)?;
    let protocol = request
        .protocol
        .as_deref()
        .unwrap_or("https")
        .to_ascii_lowercase();
    let tunnel_id = request.tunnel_id.trim();

    if !tunnel_id.is_empty() {
        state
            .edit_tunnel(
                tunnel_id.to_string(),
                UpdateTunnelRequest {
                    host: Some(host.clone()),
                    protocol: Some(protocol),
                    path: request.path,
                    ..Default::default()
                },
            )
            .await?;
    } else {
        ensure_managed_domain_record(&host, None)?;
    }

    if let Some(project_id) = request.project_id.as_deref() {
        let _ = projects.add_domain(project_id, host.clone());
    }

    domain_detail(state, projects, host).await
}

pub async fn domain_delete(state: &ClientRuntimeState, host: String) -> Result<Value, String> {
    let normalized = normalize_host(&host)?;
    let items = collect_domain_items(state, &ProjectWorkspaceState::default()).await?;
    let tunnel_id = items
        .iter()
        .find(|item| item.get("host").and_then(Value::as_str) == Some(normalized.as_str()))
        .and_then(|item| item.get("tunnelId").and_then(Value::as_str))
        .map(str::to_string)
        .ok_or_else(|| "DOMAIN_NOT_FOUND".to_string())?;

    state
        .edit_tunnel(
            tunnel_id,
            UpdateTunnelRequest {
                host: Some(String::new()),
                ..Default::default()
            },
        )
        .await?;

    Ok(json!({
        "host": normalized,
        "deleted": true,
        "generatedAt": Utc::now().timestamp_millis(),
    }))
}

pub async fn domain_bind_tunnel(
    state: &ClientRuntimeState,
    request: DomainBindRequest,
) -> Result<Value, String> {
    let host = normalize_host(&request.host)?;
    state
        .edit_tunnel(
            request.tunnel_id,
            UpdateTunnelRequest {
                host: Some(host.clone()),
                ..Default::default()
            },
        )
        .await?;
    Ok(json!({ "host": host, "bound": true }))
}

pub async fn domain_unbind_tunnel(state: &ClientRuntimeState, host: String) -> Result<Value, String> {
    domain_delete(state, host).await
}

pub async fn domain_batch(
    state: &ClientRuntimeState,
    projects: &ProjectWorkspaceState,
    request: DomainBatchRequest,
) -> Result<Value, String> {
    let mut results = Vec::new();
    for host in request.hosts {
        let outcome = match request.action.as_str() {
            "delete" => domain_delete(state, host.clone())
                .await
                .map(|_| json!({ "host": host, "ok": true })),
            "enable" | "disable" => {
                let items = collect_domain_items(state, projects).await?;
                let tunnel_id = items
                    .iter()
                    .find(|item| item.get("host").and_then(Value::as_str) == Some(host.as_str()))
                    .and_then(|item| item.get("tunnelId").and_then(Value::as_str))
                    .map(str::to_string);
                if let Some(tunnel_id) = tunnel_id {
                    if request.action == "enable" {
                        state.start_tunnel(tunnel_id).await.map(|_| {
                            json!({ "host": host, "ok": true, "action": "enable" })
                        })
                    } else {
                        state.stop_tunnel(tunnel_id).await.map(|_| {
                            json!({ "host": host, "ok": true, "action": "disable" })
                        })
                    }
                } else {
                    Err("DOMAIN_NOT_FOUND".to_string())
                }
            }
            "rebindTunnel" => {
                let tunnel_id = request
                    .tunnel_id
                    .clone()
                    .ok_or_else(|| "TUNNEL_ID_REQUIRED".to_string())?;
                domain_bind_tunnel(
                    state,
                    DomainBindRequest {
                        host: host.clone(),
                        tunnel_id,
                    },
                )
                .await
                .map(|_| json!({ "host": host, "ok": true, "action": "rebindTunnel" }))
            }
            "checkDns" => domain_check_dns(state, host.clone())
                .await
                .map(|dns| json!({ "host": host, "ok": true, "dns": dns })),
            _ => Err(format!("UNSUPPORTED_BATCH_ACTION:{}", request.action)),
        };

        results.push(match outcome {
            Ok(value) => value,
            Err(error) => json!({ "host": host, "ok": false, "error": error }),
        });
    }

    let _ = request.certificate_domain;
    Ok(json!({ "results": results, "generatedAt": Utc::now().timestamp_millis() }))
}

pub async fn domain_topology(
    state: &ClientRuntimeState,
    projects: &ProjectWorkspaceState,
) -> Result<Value, String> {
    let items = collect_domain_items(state, projects).await?;
    let mut node_map: HashMap<String, Value> = HashMap::new();
    let mut edges: Vec<Value> = Vec::new();

    for item in &items {
        let host = item.get("host").and_then(Value::as_str).unwrap_or_default();
        let tunnel_id = item.get("tunnelId").and_then(Value::as_str);
        let project_id = item.get("projectId").and_then(Value::as_str);
        let cert_domain = item
            .get("certificate")
            .and_then(|cert| cert.get("domain"))
            .and_then(Value::as_str);

        node_map.insert(
            format!("domain:{host}"),
            json!({ "id": format!("domain:{host}"), "type": "domain", "label": host, "route": format!("/domains?host={host}") }),
        );

        if let Some(project_id) = project_id.filter(|value| !value.is_empty()) {
            let project_name = item
                .get("projectName")
                .and_then(Value::as_str)
                .unwrap_or(project_id);
            node_map.insert(
                format!("project:{project_id}"),
                json!({ "id": format!("project:{project_id}"), "type": "project", "label": project_name, "route": format!("/projects/{project_id}") }),
            );
            edges.push(json!({ "from": format!("project:{project_id}"), "to": format!("domain:{host}") }));
        }

        if let Some(tunnel_id) = tunnel_id.filter(|value| !value.is_empty()) {
            let tunnel_name = item
                .get("tunnelName")
                .and_then(Value::as_str)
                .unwrap_or(tunnel_id);
            node_map.insert(
                format!("tunnel:{tunnel_id}"),
                json!({ "id": format!("tunnel:{tunnel_id}"), "type": "tunnel", "label": tunnel_name, "route": format!("/tunnels/{tunnel_id}") }),
            );
            edges.push(json!({ "from": format!("tunnel:{tunnel_id}"), "to": format!("domain:{host}") }));
        }

        if item.get("https").and_then(Value::as_bool) == Some(true) {
            node_map.insert(
                format!("https:{host}"),
                json!({ "id": format!("https:{host}"), "type": "https", "label": "HTTPS", "route": format!("/domains?host={host}") }),
            );
            edges.push(json!({ "from": format!("domain:{host}"), "to": format!("https:{host}") }));
        }

        if let Some(cert_domain) = cert_domain {
            node_map.insert(
                format!("certificate:{cert_domain}"),
                json!({ "id": format!("certificate:{cert_domain}"), "type": "certificate", "label": cert_domain, "route": format!("/certificates?domain={cert_domain}") }),
            );
            edges.push(json!({ "from": format!("domain:{host}"), "to": format!("certificate:{cert_domain}") }));
        }
    }

    let nodes = node_map.into_values().collect::<Vec<_>>();
    Ok(json!({ "nodes": nodes, "edges": edges, "generatedAt": Utc::now().timestamp_millis() }))
}

async fn collect_domain_items(
    state: &ClientRuntimeState,
    projects: &ProjectWorkspaceState,
) -> Result<Vec<Value>, String> {
    let dashboard = state.dashboard().await;
    let tunnels = dashboard
        .get("tunnels")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let certificates = load_certificate_index();
    let project_index = build_project_index(projects)?;
    let managed_hosts = load_managed_hosts()?;
    let now = Utc::now().timestamp_millis();
    let cutoff = now - DAY_MS;

    let mut hosts = HashSet::new();
    for tunnel in &tunnels {
        if let Some(host) = tunnel
            .get("host")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
        {
            hosts.insert(host.to_string());
        }
    }
    for host in managed_hosts.keys() {
        hosts.insert(host.clone());
    }

    let mut items = Vec::new();
    for host in hosts {
        let tunnel = tunnels.iter().find(|tunnel| {
            tunnel.get("host").and_then(Value::as_str) == Some(host.as_str())
        });
        let tunnel_id = tunnel
            .and_then(|value| value.get("id").and_then(Value::as_str))
            .unwrap_or_default()
            .to_string();
        let protocol = tunnel
            .and_then(|value| value.get("protocol").and_then(Value::as_str))
            .unwrap_or("https")
            .to_string();
        let https = protocol == "https";
        let certificate = certificates.get(&host).cloned();
        let (request_count_24h, traffic_24h, last_access_at, request_trend, traffic_trend) =
            tunnel_metrics_for_host(tunnel, &host, cutoff, now);
        let dns_status = managed_hosts
            .get(&host)
            .cloned()
            .unwrap_or_else(|| "notChecked".to_string());
        let health_status = compute_health_status(tunnel, https, &certificate, &dns_status);
        let project = project_index.get(&host);

        items.push(json!({
            "id": domain_id_for_host(&host),
            "host": host,
            "aliases": managed_hosts_aliases(&host),
            "protocol": protocol,
            "path": tunnel.and_then(|value| value.get("path").and_then(Value::as_str)).unwrap_or("/"),
            "tunnelId": tunnel_id,
            "tunnelName": tunnel.and_then(|value| value.get("name").and_then(Value::as_str)).unwrap_or_default(),
            "projectId": project.map(|(id, _)| id.clone()).unwrap_or_default(),
            "projectName": project.map(|(_, name)| name.clone()).unwrap_or_default(),
            "serverId": tunnel.and_then(|value| value.get("serverId").and_then(Value::as_str)).unwrap_or_default(),
            "serverName": tunnel.and_then(|value| value.get("serverName").and_then(Value::as_str)).unwrap_or_default(),
            "serverHost": tunnel.and_then(|value| value.get("publicHost").and_then(Value::as_str)).unwrap_or_default(),
            "https": https,
            "certificate": certificate,
            "dnsStatus": dns_status,
            "healthStatus": health_status,
            "lastAccessAt": last_access_at,
            "requestCount24h": request_count_24h,
            "traffic24h": traffic_24h,
            "requestTrend": request_trend,
            "trafficTrend": traffic_trend,
            "createdAt": now,
            "updatedAt": now,
            "status": tunnel.and_then(|value| value.get("status").and_then(Value::as_str)).unwrap_or("unknown"),
            "url": build_public_url(&protocol, &host, tunnel.and_then(|value| value.get("path").and_then(Value::as_str))),
        }));
    }

    Ok(items)
}

fn tunnel_metrics_for_host(
    tunnel: Option<&Value>,
    host: &str,
    cutoff: i64,
    now: i64,
) -> (u64, u64, Option<i64>, Vec<u64>, Vec<u64>) {
    let bucket_ms = DAY_MS / 12;
    let mut request_trend = vec![0u64; 12];
    let mut traffic_trend = vec![0u64; 12];
    let mut request_count = 0u64;
    let mut traffic = 0u64;
    let mut last_access = None::<i64>;

    let Some(tunnel) = tunnel else {
        return (0, 0, None, request_trend, traffic_trend);
    };

    let requests = tunnel
        .get("recentRequests")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    for request in requests {
        let request_host = request.get("host").and_then(Value::as_str).unwrap_or_default();
        let timestamp = request.get("timestamp").and_then(Value::as_i64).unwrap_or(0);
        if request_host != host || timestamp < cutoff {
            continue;
        }
        request_count = request_count.saturating_add(1);
        let bytes = request.get("trafficBytes").and_then(Value::as_u64).unwrap_or(0);
        traffic = traffic.saturating_add(bytes);
        last_access = Some(last_access.map_or(timestamp, |value| value.max(timestamp)));
        let index = ((timestamp - cutoff).max(0) as f64 / bucket_ms as f64).floor() as usize;
        if index < request_trend.len() {
            request_trend[index] = request_trend[index].saturating_add(1);
            traffic_trend[index] = traffic_trend[index].saturating_add(bytes);
        }
    }

    if request_count == 0 {
        request_count = tunnel.get("requestCount").and_then(Value::as_u64).unwrap_or(0);
        traffic = tunnel.get("trafficBytes").and_then(Value::as_u64).unwrap_or(0);
        if request_count > 0 {
            last_access = Some(now);
            request_trend[11] = request_count;
            traffic_trend[11] = traffic;
        }
    }

    (request_count, traffic, last_access, request_trend, traffic_trend)
}

fn compute_health_status(
    tunnel: Option<&Value>,
    https: bool,
    certificate: &Option<Value>,
    dns_status: &str,
) -> &'static str {
    if matches!(dns_status, "mismatched" | "noRecord" | "error") {
        return "dnsError";
    }

    let Some(tunnel) = tunnel else {
        return "offline";
    };

    let status = tunnel.get("status").and_then(Value::as_str).unwrap_or("unknown");
    if !matches!(status, "running" | "starting") {
        return "tunnelOffline";
    }

    if https {
        if let Some(cert) = certificate {
            let cert_status = cert.get("status").and_then(Value::as_str).unwrap_or("unknown");
            let days = cert.get("daysRemaining").and_then(Value::as_i64).unwrap_or(0);
            if cert_status == "expired" || days <= 0 {
                return "expired";
            }
            if cert_status != "active" && cert_status != "expiringSoon" {
                return "certificateError";
            }
            if days <= 30 {
                return "warning";
            }
        } else {
            return "certificateError";
        }
    }

    if matches!(dns_status, "notChecked" | "unknown") {
        return "warning";
    }

    "healthy"
}

fn aggregate_trend(items: &[Value], key: &str) -> Vec<u64> {
    items
        .iter()
        .filter_map(|item| item.get(key).and_then(Value::as_array))
        .fold(vec![0u64; 12], |mut acc, trend| {
            for (index, value) in trend.iter().enumerate().take(acc.len()) {
                acc[index] = acc[index].saturating_add(value.as_u64().unwrap_or(0));
            }
            acc
        })
}

fn filter_domains(items: Vec<Value>, query: &DomainListQuery) -> Vec<Value> {
    let keyword = query
        .keyword
        .as_deref()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let health = query.health.as_deref().filter(|value| !value.is_empty() && *value != "all");
    let protocol = query
        .protocol
        .as_deref()
        .filter(|value| !value.is_empty() && *value != "all");

    items
        .into_iter()
        .filter(|item| {
            if let Some(keyword) = &keyword {
                let haystack = format!(
                    "{} {} {} {}",
                    item.get("host").and_then(Value::as_str).unwrap_or_default(),
                    item.get("tunnelName").and_then(Value::as_str).unwrap_or_default(),
                    item.get("projectName").and_then(Value::as_str).unwrap_or_default(),
                    item.get("serverName").and_then(Value::as_str).unwrap_or_default(),
                )
                .to_ascii_lowercase();
                if !haystack.contains(keyword) {
                    return false;
                }
            }
            if let Some(health) = health {
                if item.get("healthStatus").and_then(Value::as_str) != Some(health) {
                    return false;
                }
            }
            if let Some(protocol) = protocol {
                let https = item.get("https").and_then(Value::as_bool).unwrap_or(false);
                if protocol == "https" && !https {
                    return false;
                }
                if protocol == "http" && https {
                    return false;
                }
            }
            true
        })
        .collect()
}

fn sort_domains(mut items: Vec<Value>, sort_by: Option<&str>, sort_dir: Option<&str>) -> Vec<Value> {
    let desc = sort_dir.map(|value| value.eq_ignore_ascii_case("desc")).unwrap_or(false);
    let key = sort_by.unwrap_or("host");
    items.sort_by(|left, right| {
        let ordering = compare_json_field(left, right, key);
        if desc {
            ordering.reverse()
        } else {
            ordering
        }
    });
    items
}

fn compare_json_field(left: &Value, right: &Value, key: &str) -> std::cmp::Ordering {
    field_value(left, key).cmp(&field_value(right, key))
}

fn field_value(item: &Value, key: &str) -> String {
    match key {
        "requestCount24h" => item
            .get("requestCount24h")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .to_string(),
        "traffic24h" => item
            .get("traffic24h")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .to_string(),
        "lastAccessAt" => item
            .get("lastAccessAt")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .to_string(),
        "createdAt" => item
            .get("createdAt")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .to_string(),
        _ => item
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_ascii_lowercase(),
    }
}

async fn resolve_dns_snapshot(host: &str, server_addresses: &[String]) -> Value {
    let a_values = resolve_host_addresses(host).await;
    let resolved_to_server = a_values
        .iter()
        .any(|value| server_addresses.iter().any(|expected| expected == value));
    let status = if a_values.is_empty() {
        "noRecord"
    } else if resolved_to_server {
        "matched"
    } else {
        "mismatched"
    };

    json!({
        "host": host,
        "status": status,
        "resolvedToServer": resolved_to_server,
        "serverAddresses": server_addresses,
        "records": [{
            "type": "A",
            "values": a_values,
            "ttl": 300,
            "resolvedToServer": resolved_to_server,
            "status": status,
        }],
        "checkedAt": Utc::now().timestamp_millis(),
    })
}

async fn resolve_host_addresses(host: &str) -> Vec<String> {
    let lookup = format!("{host}:0");
    let resolved = tokio::time::timeout(Duration::from_secs(4), lookup_host(lookup.as_str())).await;
    match resolved {
        Ok(Ok(addresses)) => addresses
            .map(|addr| addr.ip().to_string())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

fn build_runtime_detail(tunnel: Option<&Value>, host: &str) -> Value {
    let Some(tunnel) = tunnel else {
        return json!({
            "httpRequests": 0,
            "trafficBytes": 0,
            "latencyMs": 0,
            "errorRate": 0,
            "currentConnections": 0,
            "peakConnections": 0,
            "requestTrend": [],
            "latencyTrend": [],
        });
    };

    let requests = tunnel
        .get("recentRequests")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|request| request.get("host").and_then(Value::as_str) == Some(host))
        .collect::<Vec<_>>();

    let request_count = if requests.is_empty() {
        tunnel.get("requestCount").and_then(Value::as_u64).unwrap_or(0)
    } else {
        requests.len() as u64
    };
    let success_count = if requests.is_empty() {
        (request_count as f64
            * tunnel
                .get("successRate")
                .and_then(Value::as_f64)
                .unwrap_or(1.0)) as u64
    } else {
        requests
            .iter()
            .filter(|request| request.get("status").and_then(Value::as_u64).unwrap_or(500) < 400)
            .count() as u64
    };
    let latency_ms = if request_count == 0 {
        0.0
    } else if requests.is_empty() {
        tunnel
            .get("averageResponseTimeMs")
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
    } else {
        requests
            .iter()
            .map(|request| request.get("latencyMs").and_then(Value::as_u64).unwrap_or(0) as f64)
            .sum::<f64>()
            / request_count as f64
    };
    let error_rate = if request_count == 0 {
        0.0
    } else {
        (request_count.saturating_sub(success_count)) as f64 / request_count as f64
    };

    json!({
        "httpRequests": request_count,
        "trafficBytes": tunnel.get("trafficBytes").and_then(Value::as_u64).unwrap_or(0),
        "latencyMs": latency_ms,
        "errorRate": error_rate,
        "currentConnections": tunnel.get("connections").and_then(Value::as_u64).unwrap_or(0),
        "peakConnections": tunnel.get("connections").and_then(Value::as_u64).unwrap_or(0),
        "requestTrend": requests.iter().map(|request| request.get("timestamp").cloned().unwrap_or(Value::Null)).collect::<Vec<_>>(),
        "latencyTrend": requests.iter().map(|request| request.get("latencyMs").cloned().unwrap_or(Value::Null)).collect::<Vec<_>>(),
    })
}

fn filter_domain_logs(logs: &Value, tunnel_id: Option<&str>, host: &str) -> Value {
    let entries = logs.as_array().cloned().unwrap_or_default();
    let access = entries
        .iter()
        .filter(|log| {
            log.get("message")
                .and_then(Value::as_str)
                .map(|message| message.to_ascii_lowercase().contains(&host.to_ascii_lowercase()))
                .unwrap_or(false)
                || log.get("tunnelId").and_then(Value::as_str) == tunnel_id
                || log.get("source").and_then(Value::as_str) == Some("http")
        })
        .take(120)
        .cloned()
        .collect::<Vec<_>>();
    let errors = access
        .iter()
        .filter(|log| log.get("level").and_then(Value::as_str) == Some("error"))
        .cloned()
        .collect::<Vec<_>>();
    json!({ "access": access, "error": errors })
}

fn load_certificate_index() -> HashMap<String, Value> {
    let mut index = HashMap::new();
    let root = certificate_store_root();
    if !root.exists() {
        return index;
    }
    let Ok(entries) = fs::read_dir(&root) else {
        return index;
    };
    for entry in entries.flatten() {
        if !entry.file_type().map(|value| value.is_dir()).unwrap_or(false) {
            continue;
        }
        let metadata_path = entry.path().join("metadata.json");
        if !metadata_path.exists() {
            continue;
        }
        let Ok(content) = fs::read_to_string(&metadata_path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&content) else {
            continue;
        };
        let Some(domain) = value.get("domain").and_then(Value::as_str) else {
            continue;
        };
        let expire_time = value
            .get("expire_time")
            .or_else(|| value.get("expireTime"))
            .and_then(parse_timestamp)
            .unwrap_or(0);
        let days_remaining = ((expire_time - Utc::now().timestamp_millis()) as f64 / 86_400_000.0)
            .ceil() as i64;
        let san = value
            .get("san")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        index.insert(
            domain.to_string(),
            json!({
                "domain": domain,
                "issuer": value.get("issuer").and_then(Value::as_str).unwrap_or_default(),
                "status": value.get("status").and_then(Value::as_str).unwrap_or("unknown"),
                "expireTime": expire_time,
                "daysRemaining": days_remaining,
                "san": san,
                "tlsVersion": value.get("tls_version").or_else(|| value.get("tlsVersion")).and_then(Value::as_str),
                "autoRenewalEnabled": value.get("auto_renewal_enabled").or_else(|| value.get("autoRenewalEnabled")).and_then(Value::as_bool).unwrap_or(false),
                "renewTime": value.get("renew_time").or_else(|| value.get("renewTime")).and_then(parse_timestamp),
            }),
        );
    }
    index
}

fn build_project_index(
    projects: &ProjectWorkspaceState,
) -> Result<HashMap<String, (String, String)>, String> {
    let mut index = HashMap::new();
    for project in projects.list()? {
        for domain in project.domains {
            index.insert(domain, (project.id.clone(), project.name.clone()));
        }
    }
    Ok(index)
}

fn load_managed_hosts() -> Result<HashMap<String, String>, String> {
    let mut hosts = HashMap::new();
    let repository = SqliteDomainRepository::open(domain_store_path()).map_err(|error| error.to_string())?;
    for domain in repository.list().map_err(|error| error.to_string())? {
        hosts.insert(
            domain.host().as_str().to_string(),
            format_dns_status(domain.dns_status()),
        );
    }
    Ok(hosts)
}

fn managed_hosts_aliases(host: &str) -> Vec<String> {
    let Ok(repository) = SqliteDomainRepository::open(domain_store_path()) else {
        return Vec::new();
    };
    let Some(managed_host) = ManagedHost::new(host).ok() else {
        return Vec::new();
    };
    let Ok(Some(domain)) = repository.find_by_host(&managed_host) else {
        return Vec::new();
    };
    domain
        .aliases()
        .iter()
        .map(|alias| alias.as_str().to_string())
        .collect()
}

fn load_managed_domain(host: &str) -> Option<Value> {
    let repository = SqliteDomainRepository::open(domain_store_path()).ok()?;
    let managed_host = ManagedHost::new(host).ok()?;
    let domain = repository.find_by_host(&managed_host).ok().flatten()?;
    Some(json!({
        "id": domain.id().as_str(),
        "recordType": format!("{:?}", domain.record_type()).to_ascii_lowercase(),
        "verifyStatus": format!("{:?}", domain.verify_status()),
        "bindStatus": format!("{:?}", domain.bind_status()),
        "resolveStatus": format!("{:?}", domain.resolve_status()),
        "dnsStatus": format_dns_status(domain.dns_status()),
        "status": format!("{:?}", domain.status()),
    }))
}

fn format_dns_status(status: &DnsStatus) -> String {
    match status {
        DnsStatus::Matched => "matched".to_string(),
        DnsStatus::Mismatched => "mismatched".to_string(),
        DnsStatus::NoRecord => "noRecord".to_string(),
        DnsStatus::NotChecked => "notChecked".to_string(),
        DnsStatus::Error(_) => "error".to_string(),
    }
}

fn server_address_map(dashboard: &Value) -> Vec<String> {
    dashboard
        .get("tunnels")
        .and_then(Value::as_array)
        .map(|tunnels| {
            tunnels
                .iter()
                .filter_map(|tunnel| tunnel.get("publicHost").and_then(Value::as_str))
                .map(str::to_string)
                .collect::<HashSet<_>>()
                .into_iter()
                .collect()
        })
        .unwrap_or_default()
}

fn build_public_url(protocol: &str, host: &str, path: Option<&str>) -> String {
    let normalized_path = path.unwrap_or("/");
    let path = if normalized_path.starts_with('/') {
        normalized_path.to_string()
    } else {
        format!("/{normalized_path}")
    };
    format!("{protocol}://{host}{path}")
}

fn domain_id_for_host(host: &str) -> String {
    host.trim().to_ascii_lowercase()
}

fn ensure_managed_domain_record(host: &str, tunnel_id: Option<&str>) -> Result<(), String> {
    let repository =
        SqliteDomainRepository::open(domain_store_path()).map_err(|error| error.to_string())?;
    let managed_host = ManagedHost::new(host).map_err(|error| error.to_string())?;
    if repository
        .find_by_host(&managed_host)
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Ok(());
    }

    let domain_id =
        DomainId::new(domain_id_for_host(host)).map_err(|error| error.to_string())?;
    let mut builder = ManagedDomain::builder(domain_id, managed_host);
    if let Some(tunnel_id) = tunnel_id.filter(|value| !value.trim().is_empty()) {
        builder = builder.tunnel_id(
            TunnelId::new(tunnel_id).map_err(|error| error.to_string())?,
        );
    }

    repository
        .create(builder.build().map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn normalize_host(host: &str) -> Result<String, String> {
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty() {
        return Err("DOMAIN_HOST_INVALID".to_string());
    }
    Ok(host)
}

fn parse_timestamp(value: &Value) -> Option<i64> {
    if let Some(number) = value.as_i64() {
        return Some(number);
    }
    value
        .as_str()
        .and_then(|text| chrono::DateTime::parse_from_rfc3339(text).ok().map(|parsed| parsed.timestamp_millis()))
}
