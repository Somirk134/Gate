use futures::future::BoxFuture;
use futures::stream::{FuturesUnordered, StreamExt};
use gate_engine::runtime::{
    HttpRouteConfig, HttpRuntimeConfig, HttpTunnelRuntime, HttpsCertificateProvider, HttpsError,
    HttpsTlsConfig, HttpsTunnelRuntime, RuntimeConfig, TimeoutConfig, TunnelRuntime,
};
use gate_server_tls::certificate::CertificateParser;
use gate_server_tls::CertificateMaterial;
use parking_lot::RwLock;
use rcgen::generate_simple_self_signed;
use rustls::crypto::ring;
use rustls::pki_types::{CertificateDer, ServerName};
use rustls::{ClientConfig, RootCertStore};
use std::collections::HashMap;
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsConnector;

const CONNECTION_MATRIX: [usize; 3] = [1000, 5000, 10000];

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "explicit runtime reliability stress test"]
async fn stress_tcp_short_connection_matrix() -> anyhow::Result<()> {
    for connections in CONNECTION_MATRIX {
        run_tcp_short_connections(connections).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "explicit runtime reliability stress test"]
async fn stress_tcp_long_connection_matrix() -> anyhow::Result<()> {
    for connections in CONNECTION_MATRIX {
        run_tcp_long_connections(connections).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "explicit runtime reliability stress test"]
async fn stress_http_keep_alive_matrix() -> anyhow::Result<()> {
    for connections in CONNECTION_MATRIX {
        run_http_keep_alive(connections).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "explicit runtime reliability stress test"]
async fn stress_https_tls_keep_alive_matrix() -> anyhow::Result<()> {
    for connections in CONNECTION_MATRIX {
        run_https_tls_keep_alive(connections).await?;
    }
    Ok(())
}

async fn run_tcp_short_connections(connections: usize) -> anyhow::Result<()> {
    let (target_addr, target_task) = spawn_echo_server().await?;
    let runtime = tcp_runtime(target_addr, connections)?;
    runtime.start().await?;
    let listen_addr = runtime.bound_addr().expect("runtime listener address");
    let started = Instant::now();

    let mut tasks = FuturesUnordered::new();
    for _ in 0..connections {
        tasks.push(async move {
            let mut stream = TcpStream::connect(listen_addr).await?;
            stream.write_all(b"gate").await?;
            let mut echoed = [0_u8; 4];
            stream.read_exact(&mut echoed).await?;
            anyhow::ensure!(&echoed == b"gate");
            Ok::<(), anyhow::Error>(())
        });
    }

    while let Some(result) = tasks.next().await {
        result?;
    }

    eprintln!(
        "tcp short connections={connections} elapsed_ms={}",
        started.elapsed().as_millis()
    );
    runtime.shutdown().await?;
    target_task.abort();
    Ok(())
}

async fn run_tcp_long_connections(connections: usize) -> anyhow::Result<()> {
    let (target_addr, target_task) = spawn_echo_server().await?;
    let runtime = tcp_runtime(target_addr, connections)?;
    runtime.start().await?;
    let listen_addr = runtime.bound_addr().expect("runtime listener address");
    let started = Instant::now();

    let mut streams = Vec::with_capacity(connections);
    for _ in 0..connections {
        streams.push(TcpStream::connect(listen_addr).await?);
    }
    for stream in &mut streams {
        stream.write_all(b"hold").await?;
        let mut echoed = [0_u8; 4];
        stream.read_exact(&mut echoed).await?;
    }

    eprintln!(
        "tcp long connections={connections} elapsed_ms={}",
        started.elapsed().as_millis()
    );
    drop(streams);
    runtime.shutdown().await?;
    target_task.abort();
    Ok(())
}

async fn run_http_keep_alive(connections: usize) -> anyhow::Result<()> {
    let (target_addr, target_task) = spawn_http_server("ok").await?;
    let runtime = HttpTunnelRuntime::new(
        HttpRuntimeConfig::builder()
            .name("http-stress")
            .listen_addr("127.0.0.1:0".parse()?)
            .max_sessions(connections + 128)
            .max_tasks((connections * 2).max(1024))
            .route(HttpRouteConfig::new("main", target_addr).host("stress.local"))
            .timeout(stress_timeout())
            .build(),
    );
    runtime.start().await?;
    let listen_addr = runtime.bound_addr().expect("runtime listener address");
    let started = Instant::now();

    let mut tasks = FuturesUnordered::new();
    for _ in 0..connections {
        tasks.push(async move {
            let mut stream = TcpStream::connect(listen_addr).await?;
            stream
                .write_all(
                    b"GET / HTTP/1.1\r\nHost: stress.local\r\nConnection: keep-alive\r\n\r\n",
                )
                .await?;
            read_http_response(&mut stream).await?;
            stream
                .write_all(b"GET /two HTTP/1.1\r\nHost: stress.local\r\nConnection: close\r\n\r\n")
                .await?;
            read_http_response(&mut stream).await?;
            Ok::<(), anyhow::Error>(())
        });
    }

    while let Some(result) = tasks.next().await {
        result?;
    }

    eprintln!(
        "http keepalive connections={connections} elapsed_ms={} requests={}",
        started.elapsed().as_millis(),
        runtime.http_metrics().request_count
    );
    runtime.shutdown().await?;
    target_task.abort();
    Ok(())
}

async fn run_https_tls_keep_alive(connections: usize) -> anyhow::Result<()> {
    let material = self_signed_material("stress.local")?;
    let provider = Arc::new(MemoryCertificateProvider::from_materials(vec![
        material.clone()
    ]));
    let (target_addr, target_task) = spawn_http_server("tls").await?;
    let runtime = HttpsTunnelRuntime::builder()
        .http_config(
            HttpRuntimeConfig::builder()
                .name("https-stress")
                .listen_addr("127.0.0.1:0".parse()?)
                .max_sessions(connections + 128)
                .max_tasks((connections * 2).max(1024))
                .route(HttpRouteConfig::new("main", target_addr).host("stress.local"))
                .timeout(stress_timeout())
                .build(),
        )
        .tls_config(HttpsTlsConfig::default())
        .certificate_provider(provider)
        .build()?;
    runtime.start().await?;
    let listen_addr = runtime.bound_addr().expect("runtime listener address");
    let client_config = Arc::new(tls_client_config(&material)?);
    let started = Instant::now();

    let mut tasks = FuturesUnordered::new();
    for _ in 0..connections {
        let client_config = Arc::clone(&client_config);
        tasks.push(async move {
            let connector = TlsConnector::from(client_config);
            let stream = TcpStream::connect(listen_addr).await?;
            let server_name = ServerName::try_from("stress.local".to_string())?;
            let mut stream = connector.connect(server_name, stream).await?;
            stream
                .write_all(
                    b"GET / HTTP/1.1\r\nHost: stress.local\r\nConnection: keep-alive\r\n\r\n",
                )
                .await?;
            read_tls_http_response(&mut stream).await?;
            stream
                .write_all(b"GET /two HTTP/1.1\r\nHost: stress.local\r\nConnection: close\r\n\r\n")
                .await?;
            read_tls_http_response(&mut stream).await?;
            Ok::<(), anyhow::Error>(())
        });
    }

    while let Some(result) = tasks.next().await {
        result?;
    }

    let metrics = runtime.https_metrics();
    eprintln!(
        "https tls keepalive connections={connections} elapsed_ms={} handshakes={} requests={}",
        started.elapsed().as_millis(),
        metrics.handshake_count,
        metrics.http.request_count
    );
    runtime.shutdown().await?;
    target_task.abort();
    Ok(())
}

fn tcp_runtime(target_addr: SocketAddr, connections: usize) -> anyhow::Result<TunnelRuntime> {
    Ok(TunnelRuntime::new(
        RuntimeConfig::builder()
            .name("tcp-stress")
            .listen_addr("127.0.0.1:0".parse()?)
            .target_addr(target_addr)
            .max_sessions(connections + 128)
            .max_tasks((connections * 2).max(1024))
            .timeout(stress_timeout())
            .build(),
    ))
}

fn stress_timeout() -> TimeoutConfig {
    TimeoutConfig::builder()
        .connect_timeout(Duration::from_secs(5))
        .read_timeout(Duration::from_secs(30))
        .write_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(60))
        .shutdown_timeout(Duration::from_secs(10))
        .build()
}

async fn spawn_echo_server() -> anyhow::Result<(SocketAddr, tokio::task::JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let task = tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let (mut reader, mut writer) = stream.into_split();
                let _ = tokio::io::copy(&mut reader, &mut writer).await;
            });
        }
    });
    Ok((addr, task))
}

async fn spawn_http_server(
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
                loop {
                    if read_http_head(&mut stream).await.is_err() {
                        break;
                    }
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    if stream.write_all(response.as_bytes()).await.is_err() {
                        break;
                    }
                }
            });
        }
    });
    Ok((addr, task))
}

async fn read_http_response(stream: &mut TcpStream) -> anyhow::Result<()> {
    let head = read_http_head(stream).await?;
    let text = String::from_utf8(head)?;
    let length = content_length(&text).unwrap_or_default();
    let mut body = vec![0_u8; length];
    stream.read_exact(&mut body).await?;
    Ok(())
}

async fn read_tls_http_response(
    stream: &mut tokio_rustls::client::TlsStream<TcpStream>,
) -> anyhow::Result<()> {
    let head = read_tls_http_head(stream).await?;
    let text = String::from_utf8(head)?;
    let length = content_length(&text).unwrap_or_default();
    let mut body = vec![0_u8; length];
    stream.read_exact(&mut body).await?;
    Ok(())
}

async fn read_http_head(stream: &mut TcpStream) -> anyhow::Result<Vec<u8>> {
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

async fn read_tls_http_head(
    stream: &mut tokio_rustls::client::TlsStream<TcpStream>,
) -> anyhow::Result<Vec<u8>> {
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

fn content_length(text: &str) -> Option<usize> {
    text.lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("Content-Length"))
        .and_then(|(_, value)| value.trim().parse().ok())
}

#[derive(Debug, Default)]
struct MemoryCertificateProvider {
    materials: RwLock<HashMap<String, CertificateMaterial>>,
}

impl MemoryCertificateProvider {
    fn from_materials(materials: Vec<CertificateMaterial>) -> Self {
        let provider = Self::default();
        for material in materials {
            provider
                .materials
                .write()
                .insert(material.record.domain.clone(), material);
        }
        provider
    }
}

impl HttpsCertificateProvider for MemoryCertificateProvider {
    fn load_all<'a>(&'a self) -> BoxFuture<'a, Result<Vec<CertificateMaterial>, HttpsError>> {
        Box::pin(async move { Ok(self.materials.read().values().cloned().collect()) })
    }

    fn load<'a>(
        &'a self,
        domain: &'a str,
    ) -> BoxFuture<'a, Result<CertificateMaterial, HttpsError>> {
        Box::pin(async move {
            self.materials
                .read()
                .get(domain)
                .cloned()
                .ok_or_else(|| HttpsError::Certificate {
                    reason: format!("missing test certificate for {domain}"),
                })
        })
    }

    fn reload<'a>(
        &'a self,
        domain: &'a str,
    ) -> BoxFuture<'a, Result<CertificateMaterial, HttpsError>> {
        self.load(domain)
    }
}

fn self_signed_material(domain: &str) -> anyhow::Result<CertificateMaterial> {
    let certificate = generate_simple_self_signed(vec![domain.to_string()])?;
    let certificate_pem = certificate.serialize_pem()?;
    let private_key_pem = certificate.serialize_private_key_pem();
    let record = CertificateParser::parse_pem(domain, &certificate_pem)?;
    Ok(CertificateMaterial {
        certificate_pem,
        private_key_pem,
        record,
    })
}

fn tls_client_config(material: &CertificateMaterial) -> anyhow::Result<ClientConfig> {
    let mut roots = RootCertStore::empty();
    roots.add(first_certificate_der(&material.certificate_pem)?)?;
    let provider = ring::default_provider();
    Ok(ClientConfig::builder_with_provider(Arc::new(provider))
        .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])?
        .with_root_certificates(roots)
        .with_no_client_auth())
}

fn first_certificate_der(pem: &str) -> anyhow::Result<CertificateDer<'static>> {
    let mut reader = Cursor::new(pem.as_bytes());
    let certificate = rustls_pemfile::certs(&mut reader)
        .next()
        .transpose()?
        .ok_or_else(|| anyhow::anyhow!("missing generated certificate"))?;
    Ok(certificate)
}
