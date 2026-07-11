use std::io::{self, IsTerminal, Write};
use tracing_subscriber::EnvFilter;

/// 初始化日志：写入 stdout，并读取 `GATE_LOG` / `RUST_LOG`。
/// 容器控制台（如宝塔 Docker 面板）通常只采集 stdout，因此不能仅用默认 stderr。
pub fn init() {
    let filter = log_filter_from_env();
    let use_ansi = io::stdout().is_terminal();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(io::stdout)
        .with_ansi(use_ansi)
        .with_target(true)
        .with_level(true)
        .init();
}

fn log_filter_from_env() -> EnvFilter {
    if let Ok(value) = std::env::var("RUST_LOG") {
        return EnvFilter::new(value);
    }

    let level = std::env::var("GATE_LOG").unwrap_or_else(|_| "info".to_string());
    let level = level.trim();
    let level = if level.is_empty() { "info" } else { level };
    EnvFilter::new(format!("{level},gate_server={level},gate_gateway={level}"))
}

pub fn startup_line(message: impl AsRef<str>) {
    let _ = writeln!(io::stdout(), "[gate-server] {}", message.as_ref());
    let _ = io::stdout().flush();
}

pub fn fatal(message: impl AsRef<str>) -> ! {
    startup_line(format!("FATAL: {}", message.as_ref()));
    std::process::exit(1);
}
