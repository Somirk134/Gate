use chrono::Utc;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::{TcpListener, TcpStream},
    process::Command,
    time::Duration,
};

const SYSTEM_RESERVED_MAX: u16 = 1023;

pub fn local_service_discovery() -> Value {
    let listeners = discover_tcp_listeners();
    json!({
        "items": listeners.iter().map(local_service_from_listener).collect::<Vec<_>>(),
        "updatedAt": Utc::now().timestamp_millis()
    })
}

pub fn probe_local_service(host: &str, port: u16) -> Value {
    let normalized_host = normalize_connect_host(host);
    let listener = discover_tcp_listeners()
        .into_iter()
        .find(|row| port_of(row) == Some(port));
    let reachable = TcpStream::connect_timeout(
        &format!("{normalized_host}:{port}")
            .parse()
            .unwrap_or_else(|_| ([127, 0, 0, 1], port).into()),
        Duration::from_millis(700),
    )
    .is_ok();

    let row = listener.unwrap_or_else(|| {
        listener_json(
            host.to_string(),
            port,
            None,
            Some(if reachable {
                "Manual Service".to_string()
            } else {
                String::new()
            }),
            None,
        )
    });

    let mut service = local_service_from_listener(&row);
    service["host"] = json!(normalized_host);
    service["reachable"] = json!(reachable);
    service["manual"] = json!(true);
    service["label"] = json!(if service
        .get("technology")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .is_empty()
    {
        format!("Manual Service :{port}")
    } else {
        service
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or("Manual Service")
            .to_string()
    });
    service
}

pub fn local_port_discovery(gate_reserved: BTreeSet<u16>) -> Value {
    let occupied = discover_tcp_listeners();
    port_discovery_json(&occupied, &gate_reserved)
}

pub fn discover_tcp_listeners() -> Vec<Value> {
    if cfg!(target_os = "windows") {
        return discover_windows_listeners();
    }
    if cfg!(target_os = "macos") {
        return discover_macos_listeners();
    }
    discover_linux_listeners()
}

pub fn port_discovery_json(occupied: &[Value], gate_reserved: &BTreeSet<u16>) -> Value {
    let occupied_ports = occupied
        .iter()
        .filter(|row| port_of(row).is_some_and(|port| port > SYSTEM_RESERVED_MAX))
        .cloned()
        .collect::<Vec<_>>();
    let system_reserved_ports = occupied
        .iter()
        .filter(|row| port_of(row).is_some_and(|port| port <= SYSTEM_RESERVED_MAX))
        .cloned()
        .collect::<Vec<_>>();
    let gate_reserved_ports = occupied
        .iter()
        .filter(|row| port_of(row).is_some_and(|port| gate_reserved.contains(&port)))
        .cloned()
        .collect::<Vec<_>>();
    let occupied_set = occupied
        .iter()
        .filter_map(port_of)
        .chain(gate_reserved.iter().copied())
        .collect::<BTreeSet<_>>();

    json!({
        "occupiedPorts": occupied_ports,
        "availablePorts": recommended_available_ports(&occupied_set),
        "systemReservedPorts": system_reserved_ports,
        "gateReservedPorts": gate_reserved_ports,
        "updatedAt": Utc::now().timestamp_millis()
    })
}

pub fn recommend_remote_port(local_port: u16, occupied: &BTreeSet<u16>) -> Option<u16> {
    // 使用本地服务端口作为起点动态探测，避免维护静态端口列表。
    let seeds = [
        local_port,
        local_port.saturating_add(10_000),
        local_port.saturating_add(20_000),
    ];
    for seed in seeds {
        if seed > SYSTEM_RESERVED_MAX && !occupied.contains(&seed) {
            return Some(seed);
        }
    }
    (10_000_u16..65_000_u16).find(|port| !occupied.contains(port))
}

pub fn infer_tunnel_protocol(port: u16, process_name: &str) -> &'static str {
    let process = process_name.to_ascii_lowercase();
    if port == 443 {
        return "https";
    }
    if matches!(port, 80 | 3000 | 5173 | 8000 | 8080 | 8088 | 4200 | 5000)
        || [
            "node", "java", "vite", "react", "vue", "nginx", "apache", "spring", "gin", "express",
        ]
        .iter()
        .any(|needle| process.contains(needle))
    {
        return "http";
    }
    "tcp"
}

pub fn diagnose_start_failure(
    local_host: &str,
    local_port: u16,
    remote_port: u16,
    remote_occupied: &BTreeSet<u16>,
    connected: bool,
    last_error: Option<&str>,
) -> Value {
    let mut findings = Vec::new();

    if !connected {
        findings.push(finding(
            "server.offline",
            "error",
            "Server is offline or the token session is unavailable.",
            "Connect the selected server again and verify the token.",
        ));
    }

    if !local_port_listening(local_host, local_port) {
        findings.push(finding(
            "local.portNotListening",
            "error",
            "The selected local service is not listening.",
            "Start the local service or choose another discovered service.",
        ));
    }

    if remote_port == 0 {
        findings.push(finding(
            "remote.portMissing",
            "warning",
            "Remote port has not been allocated yet.",
            "Use Auto Allocate or choose one of the recommended available ports.",
        ));
    } else if remote_occupied.contains(&remote_port) {
        findings.push(finding(
            "remote.portOccupied",
            "error",
            "The remote port is already occupied on the server.",
            "Choose a recommended available port before starting the tunnel.",
        ));
    }

    if let Some(error) = last_error.filter(|value| !value.trim().is_empty()) {
        findings.push(finding(
            "runtime.lastError",
            "warning",
            error,
            &diagnosis_solution_for_error(error),
        ));
    }

    let ok = findings
        .iter()
        .all(|row| row.get("status").and_then(Value::as_str) != Some("error"));

    json!({
        "ok": ok,
        "summary": if ok { "Ready to start" } else { "Action is required before the tunnel can start" },
        "findings": findings,
        "checkedAt": Utc::now().timestamp_millis()
    })
}

fn local_service_from_listener(row: &Value) -> Value {
    let port = port_of(row).unwrap_or_default();
    let bind_address = row
        .get("bindAddress")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let process = row
        .get("processName")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let tech = detect_technology(process, port);
    let connect_host = normalize_connect_host(bind_address);
    json!({
        "port": port,
        "process": process,
        "protocol": "tcp",
        "executable": row.get("executable").and_then(Value::as_str).unwrap_or_default(),
        "pid": row.get("pid").cloned().unwrap_or(Value::Null),
        "technology": tech,
        "recommendedProtocol": infer_tunnel_protocol(port, process),
        "host": connect_host,
        "bindAddress": bind_address,
        "label": if tech.is_empty() { format!("TCP :{port}") } else { format!("{tech} :{port}") }
    })
}

fn discover_windows_listeners() -> Vec<Value> {
    let pid_names = windows_pid_names();
    command_output("netstat", &["-ano", "-p", "tcp"])
        .lines()
        .filter(|line| line.contains("LISTENING"))
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            let local = parts.get(1)?;
            let pid = parts.last().and_then(|value| value.parse::<u32>().ok());
            let (bind_address, port) = parse_endpoint(local)?;
            Some(listener_json(
                bind_address,
                port,
                pid,
                pid.and_then(|value| pid_names.get(&value).cloned()),
                None,
            ))
        })
        .collect()
}

fn discover_linux_listeners() -> Vec<Value> {
    let output = command_output("ss", &["-ltnp"]);
    if output.trim().is_empty() {
        return discover_netstat_unix_listeners();
    }
    output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            let local = parts.get(3)?;
            let (bind_address, port) = parse_endpoint(local)?;
            let process = parse_ss_process(line);
            Some(listener_json(
                bind_address,
                port,
                process.0,
                process.1,
                process.2,
            ))
        })
        .collect()
}

fn discover_macos_listeners() -> Vec<Value> {
    command_output("lsof", &["-nP", "-iTCP", "-sTCP:LISTEN"])
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            let process = parts.first().map(|value| (*value).to_string());
            let pid = parts.get(1).and_then(|value| value.parse::<u32>().ok());
            let (bind_address, port) = parts.iter().rev().find_map(|part| parse_endpoint(part))?;
            Some(listener_json(bind_address, port, pid, process, None))
        })
        .collect()
}

fn discover_netstat_unix_listeners() -> Vec<Value> {
    command_output("netstat", &["-ltnp"])
        .lines()
        .filter(|line| line.contains("LISTEN"))
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            let local = parts.get(3)?;
            let (bind_address, port) = parse_endpoint(local)?;
            Some(listener_json(bind_address, port, None, None, None))
        })
        .collect()
}

fn listener_json(
    bind_address: String,
    port: u16,
    pid: Option<u32>,
    process_name: Option<String>,
    executable: Option<String>,
) -> Value {
    json!({
        "bindAddress": bind_address,
        "port": port,
        "pid": pid,
        "processName": process_name.unwrap_or_default(),
        "protocol": "tcp",
        "status": "LISTEN",
        "executable": executable.unwrap_or_default()
    })
}

fn parse_endpoint(value: &str) -> Option<(String, u16)> {
    let trimmed = value.trim().trim_matches('"');
    let (host, port) = trimmed.rsplit_once(':')?;
    let port = port.parse::<u16>().ok()?;
    Some((host.trim_matches(['[', ']']).to_string(), port))
}

fn normalize_connect_host(host: &str) -> String {
    let host = host.trim().trim_matches(['[', ']']);
    if host.is_empty() || host == "0.0.0.0" || host == "::" || host == "*" {
        "127.0.0.1".to_string()
    } else {
        host.to_string()
    }
}

fn parse_ss_process(line: &str) -> (Option<u32>, Option<String>, Option<String>) {
    let Some(users) = line.split("users:").nth(1) else {
        return (None, None, None);
    };
    let name = users
        .split('"')
        .nth(1)
        .map(|value| value.to_string())
        .filter(|value| !value.is_empty());
    let pid = users
        .split("pid=")
        .nth(1)
        .and_then(|value| value.split(',').next())
        .and_then(|value| value.parse::<u32>().ok());
    (pid, name, None)
}

fn windows_pid_names() -> BTreeMap<u32, String> {
    command_output("tasklist", &["/fo", "csv", "/nh"])
        .lines()
        .filter_map(|line| {
            let columns = parse_csv_line(line);
            let name = columns.first()?.clone();
            let pid = columns.get(1)?.parse::<u32>().ok()?;
            Some((pid, name))
        })
        .collect()
}

fn parse_csv_line(line: &str) -> Vec<String> {
    line.trim_matches('"')
        .split("\",\"")
        .map(|value| value.to_string())
        .collect()
}

fn recommended_available_ports(occupied: &BTreeSet<u16>) -> Vec<Value> {
    let mut rows = Vec::new();
    for port in 10_000_u16..65_000_u16 {
        if rows.len() >= 24 {
            break;
        }
        if occupied.contains(&port) || !port_can_bind(port) {
            continue;
        }
        rows.push(json!({
            "port": port,
            "protocol": "tcp",
            "status": "available",
            "recommended": rows.len() < 6
        }));
    }
    rows
}

fn local_port_listening(host: &str, port: u16) -> bool {
    std::net::TcpStream::connect((host, port)).is_ok()
}

fn port_can_bind(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

fn port_of(value: &Value) -> Option<u16> {
    value
        .get("port")
        .and_then(Value::as_u64)
        .and_then(|port| u16::try_from(port).ok())
}

fn detect_technology(process_name: &str, port: u16) -> String {
    let process = process_name.to_ascii_lowercase();
    let checks = [
        ("spring", "Spring Boot"),
        ("java", "Spring Boot"),
        ("node", "Node"),
        ("vite", "Vite"),
        ("react", "React"),
        ("vue", "Vue Dev Server"),
        ("python", "Python"),
        ("nginx", "Nginx"),
        ("apache", "Apache"),
        ("redis", "Redis"),
        ("mysql", "MySQL"),
        ("postgres", "PostgreSQL"),
        ("mongo", "MongoDB"),
        ("docker", "Docker"),
        ("go", "Go"),
        ("gin", "Gin"),
        ("express", "Express"),
    ];
    if let Some((_, label)) = checks.iter().find(|(needle, _)| process.contains(needle)) {
        return (*label).to_string();
    }
    match port {
        3306 => "MySQL",
        5432 => "PostgreSQL",
        6379 => "Redis",
        27017 => "MongoDB",
        5173 => "Vite",
        3000 => "Node",
        8080 | 8000 => "HTTP Service",
        80 => "Web Server",
        443 => "HTTPS Service",
        _ => "",
    }
    .to_string()
}

fn finding(id: &str, status: &str, reason: &str, solution: &str) -> Value {
    json!({
        "id": id,
        "status": status,
        "reason": reason,
        "solution": solution
    })
}

fn diagnosis_solution_for_error(error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("refused") {
        "Check that the local service is running and listening on the selected port.".to_string()
    } else if lower.contains("already used") || lower.contains("occupied") {
        "Choose another remote port from the recommended list.".to_string()
    } else if lower.contains("auth") || lower.contains("token") {
        "Verify the server token and reconnect the server.".to_string()
    } else if lower.contains("certificate") || lower.contains("acme") {
        "Open Certificate Center and verify the domain certificate or ACME challenge.".to_string()
    } else if lower.contains("domain") {
        "Verify the domain DNS record points to this server.".to_string()
    } else {
        "Open logs for details, then run Health Check from the service actions.".to_string()
    }
}

fn command_output(program: &str, args: &[&str]) -> String {
    let direct = Command::new(program)
        .args(args)
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_default();
    if !direct.trim().is_empty() {
        return direct;
    }

    if cfg!(target_os = "windows") {
        return Command::new("cmd")
            .args(["/C", program])
            .args(args)
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .unwrap_or_default();
    }

    direct
}
