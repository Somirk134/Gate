use std::{
    fs::{self, File, OpenOptions},
    io::{self, IsTerminal, Write},
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};
use tracing_subscriber::{fmt::MakeWriter, EnvFilter};

const SERVER_LOG_FILE_NAME: &str = "gate-server.log";
const SERVER_LOG_ROLLING_FILE_NAME: &str = "gate-server.log.1";
const SERVER_LOG_MAX_BYTES: u64 = 20 * 1024 * 1024;

static SERVER_LOG_FILE: OnceLock<Arc<Mutex<File>>> = OnceLock::new();

/// 初始化日志：写入 stdout，并读取 `GATE_LOG` / `RUST_LOG`。
/// 容器控制台（如宝塔 Docker 面板）通常只采集 stdout，因此不能仅用默认 stderr。
pub fn init() {
    let filter = log_filter_from_env();
    let log_file = open_server_log_file();
    let use_ansi = io::stdout().is_terminal() && log_file.is_none();
    if let Some(file) = &log_file {
        let _ = SERVER_LOG_FILE.set(Arc::clone(file));
    }

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(TeeLogWriter { file: log_file })
        .with_ansi(use_ansi)
        .with_target(true)
        .with_level(true)
        .init();
}

#[derive(Clone)]
struct TeeLogWriter {
    file: Option<Arc<Mutex<File>>>,
}

struct TeeLogOutput {
    stdout: io::Stdout,
    file: Option<Arc<Mutex<File>>>,
}

impl<'a> MakeWriter<'a> for TeeLogWriter {
    type Writer = TeeLogOutput;

    fn make_writer(&'a self) -> Self::Writer {
        TeeLogOutput {
            stdout: io::stdout(),
            file: self.file.as_ref().map(Arc::clone),
        }
    }
}

impl Write for TeeLogOutput {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let written = self.stdout.write(buffer)?;
        if let Some(file) = &self.file {
            // 文件日志只是诊断辅助，不能因为落盘失败影响服务端 stdout 输出。
            if let Ok(mut file) = file.lock() {
                let _ = file.write_all(&buffer[..written]);
            }
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()?;
        if let Some(file) = &self.file {
            if let Ok(mut file) = file.lock() {
                let _ = file.flush();
            }
        }
        Ok(())
    }
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
    let line = format!("[gate-server] {}\n", message.as_ref());
    let _ = io::stdout().write_all(line.as_bytes());
    let _ = io::stdout().flush();
    if let Some(file) = SERVER_LOG_FILE.get() {
        if let Ok(mut file) = file.lock() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    }
}

pub fn fatal(message: impl AsRef<str>) -> ! {
    startup_line(format!("FATAL: {}", message.as_ref()));
    std::process::exit(1);
}

fn open_server_log_file() -> Option<Arc<Mutex<File>>> {
    let dir = server_log_dir();
    if fs::create_dir_all(&dir).is_err() {
        return None;
    }

    let path = dir.join(SERVER_LOG_FILE_NAME);
    rotate_large_log_file(&path);
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .ok()
        .map(|file| Arc::new(Mutex::new(file)))
}

fn rotate_large_log_file(path: &PathBuf) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };
    if metadata.len() < SERVER_LOG_MAX_BYTES {
        return;
    }

    let rotated = path.with_file_name(SERVER_LOG_ROLLING_FILE_NAME);
    let _ = fs::remove_file(&rotated);
    let _ = fs::rename(path, rotated);
}

fn server_log_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("GATE_SERVER_LOG_DIR") {
        return PathBuf::from(path);
    }
    if let Some(path) = std::env::var_os("GATE_DATA_DIR") {
        return PathBuf::from(path).join("logs");
    }

    platform_data_dir()
        .map(|base| base.join(app_data_dir_name()).join("logs"))
        .unwrap_or_else(|| PathBuf::from(".gate").join("logs"))
}

fn app_data_dir_name() -> &'static str {
    if cfg!(debug_assertions) {
        "Gate-dev"
    } else {
        "Gate-release"
    }
}

#[cfg(target_os = "windows")]
fn platform_data_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("LOCALAPPDATA"))
        .map(PathBuf::from)
}

#[cfg(target_os = "macos")]
fn platform_data_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library").join("Application Support"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_data_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
        })
}
