use chrono::Utc;
use gate_shared::process::hidden_command;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::{SocketAddr, TcpStream},
    time::Duration,
};
use tauri::{AppHandle, Emitter};

const SYSTEM_RESERVED_MAX: u16 = 1023;

const COMMON_DEV_PORTS: &[u16] = &[
    80, 443, 3000, 3001, 4000, 4200, 5000, 5173, 5174, 5432, 5672, 6379, 7000, 8000, 8080, 8081,
    8088, 8443, 8888, 9000, 9090, 9200, 15672, 27017,
];

fn common_dev_port_set() -> BTreeSet<u16> {
    COMMON_DEV_PORTS.iter().copied().collect()
}

fn is_common_dev_port(port: u16) -> bool {
    COMMON_DEV_PORTS.binary_search(&port).is_ok()
}

pub fn common_dev_port_count() -> usize {
    COMMON_DEV_PORTS.len()
}

pub fn local_service_discovery() -> Value {
    let listeners = dedupe_listen_ports(discover_common_dev_listeners());
    json!({
        "items": listeners.iter().map(local_service_from_listener).collect::<Vec<_>>(),
        "updatedAt": Utc::now().timestamp_millis()
    })
}

pub fn start_common_dev_port_scan(app: AppHandle, scan_id: String) {
    std::thread::spawn(move || run_common_dev_port_scan(&app, scan_id));
}

fn run_common_dev_port_scan(app: &AppHandle, scan_id: String) {
    let total = COMMON_DEV_PORTS.len();
    let mut found_ports = BTreeSet::new();

    for (index, &port) in COMMON_DEV_PORTS.iter().enumerate() {
        let listening = local_port_reachable_fast("127.0.0.1", port);
        if listening {
            found_ports.insert(port);
        }
        let _ = app.emit(
            "discovery-scan:progress",
            json!({
                "scanId": scan_id,
                "port": port,
                "found": listening,
                "index": index + 1,
                "total": total,
            }),
        );
    }

    let listeners = if found_ports.is_empty() {
        Vec::new()
    } else {
        dedupe_listen_ports(
            discover_common_dev_listeners()
                .into_iter()
                .filter(|row| port_of(row).is_some_and(|port| found_ports.contains(&port)))
                .collect(),
        )
    };
    let items: Vec<Value> = listeners.iter().map(local_service_from_listener).collect();

    let _ = app.emit(
        "discovery-scan:complete",
        json!({
            "scanId": scan_id,
            "items": items,
            "updatedAt": Utc::now().timestamp_millis(),
        }),
    );
}

pub fn probe_local_service(host: &str, port: u16) -> Value {
    let normalized_host = normalize_connect_host(host);
    let listener = discover_listener_for_port(port);
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

fn discover_common_dev_listeners() -> Vec<Value> {
    let ports = common_dev_port_set();
    if cfg!(target_os = "windows") {
        return discover_windows_listeners_for_ports(&ports);
    }
    if cfg!(target_os = "macos") {
        return discover_macos_listeners_for_ports(&ports);
    }
    discover_linux_listeners_for_ports(&ports)
}

fn discover_listener_for_port(port: u16) -> Option<Value> {
    let ports = BTreeSet::from([port]);
    if cfg!(target_os = "windows") {
        return discover_windows_listeners_for_ports(&ports)
            .into_iter()
            .next();
    }
    if cfg!(target_os = "macos") {
        return discover_macos_listeners_for_ports(&ports)
            .into_iter()
            .next();
    }
    discover_linux_listeners_for_ports(&ports)
        .into_iter()
        .next()
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
            "error",
            "Remote port has not been configured.",
            "Enter a valid remote port before starting the tunnel.",
        ));
    } else if remote_occupied.contains(&remote_port) {
        findings.push(finding(
            "remote.portOccupied",
            "error",
            "The remote port is already occupied on the server.",
            "Choose another remote port before starting the tunnel.",
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
        "label": if tech.is_empty() { format!("TCP :{port}") } else { format!("{tech} :{port}") },
        "reachable": true
    })
}

fn discover_windows_listeners() -> Vec<Value> {
    discover_windows_listeners_filtered(None)
}

fn discover_windows_listeners_for_ports(ports: &BTreeSet<u16>) -> Vec<Value> {
    discover_windows_listeners_filtered(Some(ports))
}

fn discover_windows_listeners_filtered(only_ports: Option<&BTreeSet<u16>>) -> Vec<Value> {
    let mut rows = Vec::new();
    let mut needed_pids = BTreeSet::new();

    for line in windows_netstat_output().lines() {
        if !line_is_tcp_listen(line) {
            continue;
        }
        let parts = line.split_whitespace().collect::<Vec<_>>();
        let local = match parts.get(1) {
            Some(value) => *value,
            None => continue,
        };
        let pid = parts.last().and_then(|value| value.parse::<u32>().ok());
        let (bind_address, port) = match parse_endpoint(local) {
            Some(value) => value,
            None => continue,
        };
        if port <= SYSTEM_RESERVED_MAX {
            continue;
        }
        if let Some(ports) = only_ports {
            if !ports.contains(&port) {
                continue;
            }
        }
        if let Some(pid) = pid {
            needed_pids.insert(pid);
        }
        rows.push((bind_address, port, pid));
    }

    let pid_names = windows_pid_names_for(&needed_pids);
    rows.into_iter()
        .map(|(bind_address, port, pid)| {
            listener_json(
                bind_address,
                port,
                pid,
                pid.and_then(|value| pid_names.get(&value).cloned()),
                None,
            )
        })
        .collect()
}

fn command_output(program: &str, args: &[&str]) -> String {
    read_command_stdout(hidden_command(program).args(args)).unwrap_or_default()
}

fn read_command_stdout(command: &mut std::process::Command) -> Option<String> {
    let output = command.output().ok()?;
    decode_command_bytes(&output.stdout)
        .filter(|text| !text.trim().is_empty())
        .or_else(|| decode_command_bytes(&output.stderr).filter(|text| !text.trim().is_empty()))
}

fn decode_command_bytes(bytes: &[u8]) -> Option<String> {
    if bytes.is_empty() {
        return None;
    }

    if let Ok(text) = std::str::from_utf8(bytes) {
        return Some(text.to_string());
    }

    #[cfg(windows)]
    {
        return decode_windows_console_bytes(bytes);
    }

    #[cfg(not(windows))]
    {
        Some(String::from_utf8_lossy(bytes).to_string())
    }
}

#[cfg(windows)]
fn decode_windows_console_bytes(bytes: &[u8]) -> Option<String> {
    unsafe extern "system" {
        fn MultiByteToWideChar(
            code_page: u32,
            dw_flags: u32,
            lp_multi_byte_str: *const u8,
            cb_multi_byte: i32,
            lp_wide_char_str: *mut u16,
            cch_wide_char: i32,
        ) -> i32;
        fn WideCharToMultiByte(
            code_page: u32,
            dw_flags: u32,
            lp_wide_char_str: *const u16,
            cch_wide_char: i32,
            lp_multi_byte_str: *mut u8,
            cb_multi_byte: i32,
            lp_default_char: *const u8,
            lp_used_default_char: *mut i32,
        ) -> i32;
    }

    const CP_UTF8: u32 = 65001;
    const CP_OEMCP: u32 = 1;

    for code_page in [CP_UTF8, CP_OEMCP] {
        let wide_len = unsafe {
            MultiByteToWideChar(
                code_page,
                0,
                bytes.as_ptr(),
                bytes.len() as i32,
                std::ptr::null_mut(),
                0,
            )
        };
        if wide_len <= 0 {
            continue;
        }

        let mut wide = vec![0u16; wide_len as usize];
        let converted = unsafe {
            MultiByteToWideChar(
                code_page,
                0,
                bytes.as_ptr(),
                bytes.len() as i32,
                wide.as_mut_ptr(),
                wide_len,
            )
        };
        if converted <= 0 {
            continue;
        }

        let utf8_len = unsafe {
            WideCharToMultiByte(
                CP_UTF8,
                0,
                wide.as_ptr(),
                converted,
                std::ptr::null_mut(),
                0,
                std::ptr::null(),
                std::ptr::null_mut(),
            )
        };
        if utf8_len <= 0 {
            continue;
        }

        let mut utf8 = vec![0u8; utf8_len as usize];
        let written = unsafe {
            WideCharToMultiByte(
                CP_UTF8,
                0,
                wide.as_ptr(),
                converted,
                utf8.as_mut_ptr(),
                utf8_len,
                std::ptr::null(),
                std::ptr::null_mut(),
            )
        };
        if written <= 0 {
            continue;
        }
        utf8.truncate(written as usize);
        if let Ok(text) = String::from_utf8(utf8) {
            return Some(text);
        }
    }

    Some(String::from_utf8_lossy(bytes).to_string())
}

fn windows_netstat_output() -> String {
    let powershell = read_command_stdout(hidden_command("powershell").args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "[Console]::OutputEncoding = [Text.UTF8Encoding]::UTF8; netstat -ano -p tcp",
    ]));
    if powershell
        .as_ref()
        .is_some_and(|text| !text.trim().is_empty())
    {
        return powershell.unwrap_or_default();
    }

    let direct = command_output("netstat", &["-ano", "-p", "tcp"]);
    if !direct.trim().is_empty() {
        return direct;
    }

    read_command_stdout(hidden_command("cmd").args(["/C", "netstat", "-ano", "-p", "tcp"]))
        .unwrap_or_default()
}

fn line_is_tcp_listen(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || !trimmed.to_ascii_uppercase().starts_with("TCP") {
        return false;
    }
    trimmed.to_ascii_uppercase().contains("LISTENING")
        || trimmed.contains("监听")
        || trimmed.contains("侦听")
}

fn dedupe_listen_ports(rows: Vec<Value>) -> Vec<Value> {
    let mut by_port: BTreeMap<u16, Value> = BTreeMap::new();
    for row in rows {
        let Some(port) = port_of(&row) else {
            continue;
        };
        if port <= SYSTEM_RESERVED_MAX {
            continue;
        }
        match by_port.get(&port) {
            None => {
                by_port.insert(port, row);
            }
            Some(existing) => {
                if listener_rank(&row) > listener_rank(existing) {
                    by_port.insert(port, row);
                }
            }
        }
    }
    by_port.into_values().collect()
}

fn listener_rank(row: &Value) -> u8 {
    let mut score = 0u8;
    if row
        .get("processName")
        .and_then(Value::as_str)
        .is_some_and(|name| !name.trim().is_empty())
    {
        score += 2;
    }
    let bind = row
        .get("bindAddress")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if bind == "127.0.0.1" || bind == "::1" {
        score += 1;
    }
    score
}

fn discover_linux_listeners() -> Vec<Value> {
    discover_linux_listeners_filtered(None)
}

fn discover_linux_listeners_for_ports(ports: &BTreeSet<u16>) -> Vec<Value> {
    discover_linux_listeners_filtered(Some(ports))
}

fn discover_linux_listeners_filtered(only_ports: Option<&BTreeSet<u16>>) -> Vec<Value> {
    let output = command_output("ss", &["-ltnp"]);
    if output.trim().is_empty() {
        return discover_netstat_unix_listeners_filtered(only_ports);
    }
    output
        .lines()
        .skip(1)
        .filter_map(|line| parse_linux_listener_line(line, only_ports))
        .collect()
}

fn parse_linux_listener_line(line: &str, only_ports: Option<&BTreeSet<u16>>) -> Option<Value> {
    let parts = line.split_whitespace().collect::<Vec<_>>();
    let local = parts.get(3)?;
    let (bind_address, port) = parse_endpoint(local)?;
    if port <= SYSTEM_RESERVED_MAX {
        return None;
    }
    if let Some(ports) = only_ports {
        if !ports.contains(&port) {
            return None;
        }
    }
    let process = parse_ss_process(line);
    Some(listener_json(
        bind_address,
        port,
        process.0,
        process.1,
        process.2,
    ))
}

fn discover_macos_listeners() -> Vec<Value> {
    discover_macos_listeners_filtered(None)
}

fn discover_macos_listeners_for_ports(ports: &BTreeSet<u16>) -> Vec<Value> {
    discover_macos_listeners_filtered(Some(ports))
}

fn discover_macos_listeners_filtered(only_ports: Option<&BTreeSet<u16>>) -> Vec<Value> {
    command_output("lsof", &["-nP", "-iTCP", "-sTCP:LISTEN"])
        .lines()
        .skip(1)
        .filter_map(|line| parse_macos_listener_line(line, only_ports))
        .collect()
}

fn parse_macos_listener_line(line: &str, only_ports: Option<&BTreeSet<u16>>) -> Option<Value> {
    let parts = line.split_whitespace().collect::<Vec<_>>();
    let process = parts.first().map(|value| (*value).to_string());
    let pid = parts.get(1).and_then(|value| value.parse::<u32>().ok());
    let (bind_address, port) = parts.iter().rev().find_map(|part| parse_endpoint(part))?;
    if port <= SYSTEM_RESERVED_MAX {
        return None;
    }
    if let Some(ports) = only_ports {
        if !ports.contains(&port) {
            return None;
        }
    }
    Some(listener_json(bind_address, port, pid, process, None))
}

fn discover_netstat_unix_listeners_filtered(only_ports: Option<&BTreeSet<u16>>) -> Vec<Value> {
    command_output("netstat", &["-ltnp"])
        .lines()
        .filter(|line| line.contains("LISTEN"))
        .filter_map(|line| {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            let local = parts.get(3)?;
            let (bind_address, port) = parse_endpoint(local)?;
            if port <= SYSTEM_RESERVED_MAX {
                return None;
            }
            if let Some(ports) = only_ports {
                if !ports.contains(&port) {
                    return None;
                }
            }
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

fn windows_pid_names_for(pids: &BTreeSet<u32>) -> BTreeMap<u32, String> {
    if pids.is_empty() {
        return BTreeMap::new();
    }
    if pids.len() <= 8 {
        return pids
            .iter()
            .filter_map(|pid| windows_pid_name(*pid).map(|name| (*pid, name)))
            .collect();
    }

    let all = windows_pid_names();
    pids.iter()
        .filter_map(|pid| all.get(pid).map(|name| (*pid, name.clone())))
        .collect()
}

fn windows_pid_name(pid: u32) -> Option<String> {
    let output = command_output(
        "tasklist",
        &["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"],
    );
    let line = output.lines().next()?;
    parse_csv_line(line).first().cloned()
}

fn parse_csv_line(line: &str) -> Vec<String> {
    line.trim_matches('"')
        .split("\",\"")
        .map(|value| value.to_string())
        .collect()
}

fn local_port_listening(host: &str, port: u16) -> bool {
    local_port_reachable_fast(host, port)
}

fn local_port_reachable_fast(host: &str, port: u16) -> bool {
    let host = normalize_connect_host(host);
    let address: SocketAddr = format!("{host}:{port}")
        .parse()
        .unwrap_or_else(|_| ([127, 0, 0, 1], port).into());
    TcpStream::connect_timeout(&address, Duration::from_millis(120)).is_ok()
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
        "Choose another remote port.".to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_is_tcp_listen_accepts_english_and_chinese_states() {
        assert!(line_is_tcp_listen(
            "  TCP    0.0.0.0:8080           0.0.0.0:0              LISTENING       5260"
        ));
        assert!(line_is_tcp_listen(
            "  TCP    0.0.0.0:8080           0.0.0.0:0              监听       5260"
        ));
        assert!(!line_is_tcp_listen(
            "  TCP    127.0.0.1:8080       127.0.0.1:54321        ESTABLISHED     5260"
        ));
    }

    #[test]
    fn dedupe_listen_ports_prefers_named_process() {
        let unnamed = listener_json("0.0.0.0".to_string(), 8080, Some(1), None, None);
        let named = listener_json(
            "127.0.0.1".to_string(),
            8080,
            Some(2),
            Some("java.exe".to_string()),
            None,
        );
        let rows = dedupe_listen_ports(vec![unnamed, named.clone()]);
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].get("processName").and_then(Value::as_str),
            Some("java.exe")
        );
    }

    #[test]
    fn parse_windows_netstat_sample_line() {
        let line = "  TCP    0.0.0.0:8088           0.0.0.0:0              LISTENING       15628";
        assert!(line_is_tcp_listen(line));
        let parts = line.split_whitespace().collect::<Vec<_>>();
        let local = parts.get(1).copied().expect("local endpoint");
        let (bind_address, port) = parse_endpoint(local).expect("endpoint");
        assert_eq!(bind_address, "0.0.0.0");
        assert_eq!(port, 8088);
    }

    #[test]
    fn discover_common_dev_listeners_only_returns_common_ports() {
        let listeners = dedupe_listen_ports(discover_common_dev_listeners());
        assert!(listeners
            .iter()
            .all(|row| { port_of(row).is_some_and(is_common_dev_port) }));
    }

    #[test]
    fn discover_tcp_listeners_smoke_on_machine() {
        let netstat = windows_netstat_output();
        assert!(
            !netstat.trim().is_empty(),
            "netstat output was empty: {:?}",
            netstat.chars().take(120).collect::<String>()
        );
        let discovery = local_service_discovery();
        let items = discovery
            .get("items")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(items
            .iter()
            .all(|row| { port_of(row).is_some_and(is_common_dev_port) }));
    }
}
