use gate_engine::runtime::{HttpRouteConfig, HttpRuntimeConfig, HttpTunnelRuntime, TimeoutConfig};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

#[tokio::test]
async fn http_runtime_forwards_keep_alive_requests() -> anyhow::Result<()> {
    let (target_addr, target_task) = spawn_fixed_body_server("ok").await?;
    let runtime = http_runtime(vec![HttpRouteConfig::new("main", target_addr)
        .host("example.com")
        .path_prefix("/")])?;

    runtime.start().await?;
    let listen_addr = runtime.bound_addr().expect("runtime listener address");
    let mut client = TcpStream::connect(listen_addr).await?;

    client
        .write_all(b"GET /one HTTP/1.1\r\nHost: example.com\r\nConnection: keep-alive\r\n\r\n")
        .await?;
    let first = read_response(&mut client).await?;
    assert_eq!(first.status, 200);
    assert_eq!(first.body, b"ok");

    client
        .write_all(b"GET /two HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n")
        .await?;
    let second = read_response(&mut client).await?;
    assert_eq!(second.status, 200);
    assert_eq!(second.body, b"ok");

    tokio::time::sleep(Duration::from_millis(100)).await;
    let http_metrics = runtime.http_metrics();
    assert_eq!(http_metrics.request_count, 2);
    assert_eq!(http_metrics.success_count, 2);

    runtime.shutdown().await?;
    target_task.abort();
    Ok(())
}

#[tokio::test]
async fn http_runtime_routes_by_host_and_path() -> anyhow::Result<()> {
    let (api_addr, api_task) = spawn_fixed_body_server("api").await?;
    let (admin_addr, admin_task) = spawn_fixed_body_server("admin").await?;
    let runtime = http_runtime(vec![
        HttpRouteConfig::new("api", api_addr)
            .host("api.example.com")
            .path_prefix("/"),
        HttpRouteConfig::new("admin", admin_addr)
            .host("example.com")
            .path_prefix("/admin"),
    ])?;

    runtime.start().await?;
    let listen_addr = runtime.bound_addr().expect("runtime listener address");

    let mut api_client = TcpStream::connect(listen_addr).await?;
    api_client
        .write_all(b"GET /v1 HTTP/1.1\r\nHost: api.example.com\r\nConnection: close\r\n\r\n")
        .await?;
    let api_response = read_response(&mut api_client).await?;
    assert_eq!(api_response.body, b"api");

    let mut admin_client = TcpStream::connect(listen_addr).await?;
    admin_client
        .write_all(
            b"GET /admin/settings HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n",
        )
        .await?;
    let admin_response = read_response(&mut admin_client).await?;
    assert_eq!(admin_response.body, b"admin");

    runtime.shutdown().await?;
    api_task.abort();
    admin_task.abort();
    Ok(())
}

#[tokio::test]
async fn http_runtime_streams_chunked_requests_and_responses() -> anyhow::Result<()> {
    let (received_tx, received_rx) = oneshot::channel();
    let target_listener = TcpListener::bind("127.0.0.1:0").await?;
    let target_addr = target_listener.local_addr()?;
    let target_task = tokio::spawn(async move {
        let Ok((mut stream, _)) = target_listener.accept().await else {
            return;
        };
        let mut received = Vec::new();
        let _ = read_head_from_stream(&mut stream).await;
        loop {
            let mut byte = [0_u8; 1];
            if stream.read_exact(&mut byte).await.is_err() {
                break;
            }
            received.push(byte[0]);
            if received.ends_with(b"0\r\n\r\n") {
                break;
            }
        }
        let _ = received_tx.send(received);
        let _ = stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n7\r\nchunked\r\n2\r\nok\r\n0\r\n\r\n",
            )
            .await;
    });

    let runtime = http_runtime(vec![HttpRouteConfig::new("stream", target_addr)
        .host("stream.example.com")
        .path_prefix("/stream")])?;
    runtime.start().await?;

    let mut client = TcpStream::connect(runtime.bound_addr().unwrap()).await?;
    client
        .write_all(
            b"POST /stream HTTP/1.1\r\nHost: stream.example.com\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n",
        )
        .await?;
    let response = read_response(&mut client).await?;
    assert_eq!(response.status, 200);
    assert_eq!(response.body, b"chunkedok");

    let received = received_rx.await?;
    assert!(received.starts_with(b"5\r\nhello\r\n6\r\n world\r\n"));

    runtime.shutdown().await?;
    target_task.abort();
    Ok(())
}

fn http_runtime(routes: Vec<HttpRouteConfig>) -> anyhow::Result<HttpTunnelRuntime> {
    Ok(HttpTunnelRuntime::new(
        HttpRuntimeConfig::builder()
            .name("http-runtime-test")
            .listen_addr("127.0.0.1:0".parse()?)
            .routes(routes)
            .timeout(
                TimeoutConfig::builder()
                    .connect_timeout(Duration::from_secs(1))
                    .idle_timeout(Duration::from_secs(10))
                    .shutdown_timeout(Duration::from_secs(2))
                    .build(),
            )
            .build(),
    ))
}

async fn spawn_fixed_body_server(
    body: &'static str,
) -> anyhow::Result<(SocketAddr, tokio::task::JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let task = tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let Ok(_) = read_head_from_stream(&mut stream).await else {
                    return;
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes()).await;
            });
        }
    });
    Ok((addr, task))
}

struct Response {
    status: u16,
    body: Vec<u8>,
}

async fn read_response(stream: &mut TcpStream) -> anyhow::Result<Response> {
    let head = read_head_from_stream(stream).await?;
    let text = String::from_utf8(head)?;
    let status = text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or_else(|| anyhow::anyhow!("missing status"))?
        .parse::<u16>()?;
    let content_length = header_value(&text, "Content-Length").and_then(|value| value.parse().ok());
    let chunked = header_value(&text, "Transfer-Encoding")
        .map(|value| value.eq_ignore_ascii_case("chunked"))
        .unwrap_or(false);
    let body = if let Some(length) = content_length {
        let mut body = vec![0_u8; length];
        stream.read_exact(&mut body).await?;
        body
    } else if chunked {
        read_chunked_body(stream).await?
    } else {
        Vec::new()
    };

    Ok(Response { status, body })
}

async fn read_head_from_stream(stream: &mut TcpStream) -> anyhow::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        stream.read_exact(&mut byte).await?;
        buffer.push(byte[0]);
        if buffer.ends_with(b"\r\n\r\n") {
            return Ok(buffer);
        }
    }
}

async fn read_chunked_body(stream: &mut TcpStream) -> anyhow::Result<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        let line = read_crlf_line(stream).await?;
        let size_hex = std::str::from_utf8(&line)?.trim();
        let size = u64::from_str_radix(size_hex, 16)?;
        if size == 0 {
            let _ = read_crlf_line(stream).await?;
            break;
        }
        let mut chunk = vec![0_u8; size as usize];
        stream.read_exact(&mut chunk).await?;
        body.extend_from_slice(&chunk);
        let mut crlf = [0_u8; 2];
        stream.read_exact(&mut crlf).await?;
    }
    Ok(body)
}

async fn read_crlf_line(stream: &mut TcpStream) -> anyhow::Result<Vec<u8>> {
    let mut line = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        stream.read_exact(&mut byte).await?;
        line.push(byte[0]);
        if line.ends_with(b"\r\n") {
            return Ok(line);
        }
    }
}

fn header_value(text: &str, name: &str) -> Option<String> {
    text.lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(header, _)| header.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.trim().to_string())
}
