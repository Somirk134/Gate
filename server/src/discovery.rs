use chrono::Utc;
use gate_shared::process::hidden_command;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    env,
};
use sysinfo::{Disks, Networks, System};

const SYSTEM_RESERVED_MAX: u16 = 1023;

pub fn server_capability_snapshot(gate_reserved_ports: &[u16]) -> Value {
    let mut system = System::new_all();
    system.refresh_all();
    let disks = Disks::new_with_refreshed_list();
    let networks = Networks::new_with_refreshed_list();
    let ports = discover_tcp_listeners();
    let gate_reserved = gate_reserved_ports.iter().copied().collect::<BTreeSet<_>>();

    json!({
        "serverName": hostname(),
        "hostname": hostname(),
        "os": System::long_os_version().or_else(System::name).unwrap_or_else(|| env::consts::OS.to_string()),
        "cpu": format!("{} logical cores", system.cpus().len()),
        "memory": {
            "totalBytes": system.total_memory(),
            "usedBytes": system.used_memory()
        },
        "architecture": env::consts::ARCH,
        "publicIp": env::var("GATE_PUBLIC_IP").unwrap_or_default(),
        "privateIp": local_private_ip(),
        "docker": docker_status(),
        "firewall": firewall_status(),
        "gateVersion": env!("CARGO_PKG_VERSION"),
        "runtimeVersion": rust_runtime_version(),
        "uptime": System::uptime(),
        "region": env::var("GATE_REGION").unwrap_or_default(),
        "diskUsage": disk_usage(&disks),
        "networkUsage": network_usage(&networks),
        "portDiscovery": port_discovery_json(&ports, &gate_reserved),
        "discoveredAt": Utc::now().timestamp_millis()
    })
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
    json!({
        "occupiedPorts": occupied_ports,
        "systemReservedPorts": system_reserved_ports,
        "gateReservedPorts": gate_reserved_ports,
        "updatedAt": Utc::now().timestamp_millis()
    })
}

fn discover_windows_listeners() -> Vec<Value> {
    let pid_names = windows_pid_names();
    let output = command_output("netstat", &["-ano", "-p", "tcp"]);
    output
        .lines()
        .filter(|line| line.contains("LISTENING"))
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            let local = parts.get(1)?;
            let pid = parts.last().and_then(|value| value.parse::<u32>().ok());
            let port = parse_port(local)?;
            Some(listener_json(
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
            let port = parse_port(local)?;
            let process = parse_ss_process(line);
            Some(listener_json(port, process.0, process.1, process.2))
        })
        .collect()
}

fn discover_macos_listeners() -> Vec<Value> {
    let output = command_output("lsof", &["-nP", "-iTCP", "-sTCP:LISTEN"]);
    output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            let process = parts.first().map(|value| (*value).to_string());
            let pid = parts.get(1).and_then(|value| value.parse::<u32>().ok());
            let port = parts.iter().rev().find_map(|part| parse_port(part))?;
            Some(listener_json(port, pid, process, None))
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
            let port = parse_port(local)?;
            Some(listener_json(port, None, None, None))
        })
        .collect()
}

fn listener_json(
    port: u16,
    pid: Option<u32>,
    process_name: Option<String>,
    executable: Option<String>,
) -> Value {
    json!({
        "port": port,
        "pid": pid,
        "processName": process_name.unwrap_or_default(),
        "protocol": "tcp",
        "status": "LISTEN",
        "executable": executable.unwrap_or_default()
    })
}

fn parse_port(value: &str) -> Option<u16> {
    let trimmed = value.trim().trim_matches('"');
    let (_, port) = trimmed.rsplit_once(':')?;
    port.parse::<u16>().ok()
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

fn port_of(value: &Value) -> Option<u16> {
    value
        .get("port")
        .and_then(Value::as_u64)
        .and_then(|port| u16::try_from(port).ok())
}

fn hostname() -> String {
    System::host_name()
        .or_else(|| env::var("HOSTNAME").ok())
        .or_else(|| env::var("COMPUTERNAME").ok())
        .unwrap_or_else(|| "Gate Server".to_string())
}

fn local_private_ip() -> String {
    command_output("hostname", &["-I"])
        .split_whitespace()
        .find(|value| !value.starts_with("127."))
        .unwrap_or_default()
        .to_string()
}

fn docker_status() -> Value {
    let in_container = std::path::Path::new("/.dockerenv").exists()
        || env::var("container").is_ok()
        || env::var("DOCKER_CONTAINER").is_ok();
    json!({ "detected": in_container })
}

fn firewall_status() -> Value {
    let available = if cfg!(target_os = "windows") {
        !command_output("netsh", &["advfirewall", "show", "currentprofile"]).is_empty()
    } else {
        !command_output("ufw", &["status"]).is_empty()
            || !command_output("firewall-cmd", &["--state"]).is_empty()
    };
    json!({ "detected": available })
}

fn rust_runtime_version() -> String {
    command_output("rustc", &["--version"]).trim().to_string()
}

fn disk_usage(disks: &Disks) -> Value {
    let total = disks.iter().map(|disk| disk.total_space()).sum::<u64>();
    let available = disks.iter().map(|disk| disk.available_space()).sum::<u64>();
    json!({
        "totalBytes": total,
        "usedBytes": total.saturating_sub(available),
        "availableBytes": available
    })
}

fn network_usage(networks: &Networks) -> Value {
    let received = networks
        .iter()
        .map(|(_, data)| data.received())
        .sum::<u64>();
    let transmitted = networks
        .iter()
        .map(|(_, data)| data.transmitted())
        .sum::<u64>();
    json!({
        "receivedBytes": received,
        "transmittedBytes": transmitted
    })
}

fn command_output(program: &str, args: &[&str]) -> String {
    hidden_command(program)
        .args(args)
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_default()
}
