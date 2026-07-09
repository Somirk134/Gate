use serde_json::json;
use std::{env, fs, net::SocketAddr, path::Path};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mode = env::var("GATE_E2E_LOCAL_MODE").unwrap_or_else(|_| "tcp".to_string());
    let addr = env::var("GATE_E2E_LOCAL_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:0".to_string())
        .parse::<SocketAddr>()?;
    let listener = TcpListener::bind(addr).await?;
    let bound_addr = listener.local_addr()?;

    if let Ok(path) = env::var("GATE_E2E_READY_FILE") {
        write_json(
            Path::new(&path),
            json!({
                "kind": "local-service",
                "mode": mode,
                "addr": bound_addr.to_string(),
                "port": bound_addr.port()
            }),
        )?;
    }

    loop {
        let (stream, _) = listener.accept().await?;
        let mode = mode.clone();
        tokio::spawn(async move {
            let _ = handle_connection(stream, &mode).await;
        });
    }
}

async fn handle_connection(mut stream: TcpStream, mode: &str) -> anyhow::Result<()> {
    if mode.eq_ignore_ascii_case("http") {
        let mut buffer = [0_u8; 4096];
        let _ = stream.read(&mut buffer).await?;
        let body = b"gate-e2e-http";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(response.as_bytes()).await?;
        stream.write_all(body).await?;
        stream.shutdown().await?;
        return Ok(());
    }

    let (mut reader, mut writer) = stream.split();
    tokio::io::copy(&mut reader, &mut writer).await?;
    Ok(())
}

fn write_json(path: &Path, value: serde_json::Value) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(&value)?)?;
    Ok(())
}
