use futures::future::BoxFuture;
use gate_engine::runtime::{
    HttpHostResolver, HttpRouteConfig, HttpRuntimeConfig, HttpsCertificateProvider, HttpsError,
    HttpsRuntimeMetrics, HttpsTlsConfig, HttpsTunnelRuntime, TimeoutConfig,
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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;

#[tokio::test]
async fn https_runtime_forwards_self_signed_sni_requests() -> anyhow::Result<()> {
    let material = self_signed_material("example.com")?;
    let provider = Arc::new(MemoryCertificateProvider::from_materials(vec![
        material.clone()
    ]));
    let (target_addr, target_task) = spawn_fixed_body_server("secure").await?;
    let runtime = https_runtime(
        provider,
        None,
        vec![HttpRouteConfig::new("main", target_addr)
            .host("example.com")
            .path_prefix("/")],
    )?;

    runtime.start().await?;
    let mut client = tls_client(runtime.bound_addr().unwrap(), "example.com", &material).await?;
    client
        .write_all(b"GET /one HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n")
        .await?;
    let response = read_response(&mut client).await?;

    assert_eq!(response.status, 200);
    assert_eq!(response.body, b"secure");

    tokio::time::sleep(Duration::from_millis(100)).await;
    let metrics = runtime.https_metrics();
    assert_eq!(metrics.handshake_count, 1);
    assert_eq!(metrics.handshake_error_count, 0);
    assert_eq!(metrics.http.request_count, 1);
    assert_tls_log(&metrics, "example.com");

    runtime.shutdown().await?;
    target_task.abort();
    Ok(())
}

#[tokio::test]
async fn https_runtime_routes_multiple_sni_domains() -> anyhow::Result<()> {
    let api_material = self_signed_material("api.example.com")?;
    let admin_material = self_signed_material("admin.example.com")?;
    let provider = Arc::new(MemoryCertificateProvider::from_materials(vec![
        api_material.clone(),
        admin_material.clone(),
    ]));
    let (api_addr, api_task) = spawn_fixed_body_server("api").await?;
    let (admin_addr, admin_task) = spawn_fixed_body_server("admin").await?;
    let runtime = https_runtime(
        provider,
        None,
        vec![
            HttpRouteConfig::new("api", api_addr)
                .host("api.example.com")
                .path_prefix("/"),
            HttpRouteConfig::new("admin", admin_addr)
                .host("admin.example.com")
                .path_prefix("/"),
        ],
    )?;

    runtime.start().await?;

    let mut api_client = tls_client(
        runtime.bound_addr().unwrap(),
        "api.example.com",
        &api_material,
    )
    .await?;
    api_client
        .write_all(b"GET / HTTP/1.1\r\nHost: api.example.com\r\nConnection: close\r\n\r\n")
        .await?;
    assert_eq!(read_response(&mut api_client).await?.body, b"api");

    let mut admin_client = tls_client(
        runtime.bound_addr().unwrap(),
        "admin.example.com",
        &admin_material,
    )
    .await?;
    admin_client
        .write_all(b"GET / HTTP/1.1\r\nHost: admin.example.com\r\nConnection: close\r\n\r\n")
        .await?;
    assert_eq!(read_response(&mut admin_client).await?.body, b"admin");

    let metrics = runtime.https_metrics();
    assert_eq!(metrics.handshake_count, 2);
    assert_eq!(metrics.http.success_count, 2);
    assert_eq!(metrics.certificates.len(), 2);

    runtime.shutdown().await?;
    api_task.abort();
    admin_task.abort();
    Ok(())
}

#[tokio::test]
async fn https_runtime_uses_host_resolver_adapter() -> anyhow::Result<()> {
    let material = self_signed_material("resolver.example.com")?;
    let provider = Arc::new(MemoryCertificateProvider::from_materials(vec![
        material.clone()
    ]));
    let (target_addr, target_task) = spawn_fixed_body_server("resolved").await?;
    let route = HttpRouteConfig::new("resolved", target_addr).path_prefix("/");
    let resolver = Arc::new(CountingResolver {
        tunnel_id: route.tunnel_id.to_string(),
        calls: AtomicUsize::new(0),
    });
    let runtime = https_runtime(provider, Some(resolver.clone()), vec![route])?;

    runtime.start().await?;
    let mut client = tls_client(
        runtime.bound_addr().unwrap(),
        "resolver.example.com",
        &material,
    )
    .await?;
    client
        .write_all(
            b"GET /domain HTTP/1.1\r\nHost: resolver.example.com\r\nConnection: close\r\n\r\n",
        )
        .await?;
    assert_eq!(read_response(&mut client).await?.body, b"resolved");
    assert_eq!(resolver.calls.load(Ordering::SeqCst), 1);

    runtime.shutdown().await?;
    target_task.abort();
    Ok(())
}

#[tokio::test]
async fn https_runtime_reloads_certificate_without_restart() -> anyhow::Result<()> {
    let initial = self_signed_material("reload.example.com")?;
    let replacement = self_signed_material("reload.example.com")?;
    let initial_fingerprint = initial.record.fingerprint.sha256.clone();
    let replacement_fingerprint = replacement.record.fingerprint.sha256.clone();
    assert_ne!(initial_fingerprint, replacement_fingerprint);

    let provider = Arc::new(MemoryCertificateProvider::from_materials(vec![
        initial.clone()
    ]));
    let (target_addr, target_task) = spawn_fixed_body_server("reload").await?;
    let runtime = https_runtime(
        provider.clone(),
        None,
        vec![HttpRouteConfig::new("reload", target_addr)
            .host("reload.example.com")
            .path_prefix("/")],
    )?;

    runtime.start().await?;
    assert_certificate_fingerprint(&runtime.https_metrics(), &initial_fingerprint);

    provider.upsert(replacement.clone());
    runtime.reload_certificate("reload.example.com").await?;
    assert_certificate_fingerprint(&runtime.https_metrics(), &replacement_fingerprint);

    let mut client = tls_client(
        runtime.bound_addr().unwrap(),
        "reload.example.com",
        &replacement,
    )
    .await?;
    client
        .write_all(b"GET / HTTP/1.1\r\nHost: reload.example.com\r\nConnection: close\r\n\r\n")
        .await?;
    assert_eq!(read_response(&mut client).await?.body, b"reload");

    runtime.shutdown().await?;
    target_task.abort();
    Ok(())
}

#[derive(Debug, Default)]
struct MemoryCertificateProvider {
    materials: RwLock<HashMap<String, CertificateMaterial>>,
}

impl MemoryCertificateProvider {
    fn from_materials(materials: Vec<CertificateMaterial>) -> Self {
        let provider = Self::default();
        for material in materials {
            provider.upsert(material);
        }
        provider
    }

    fn upsert(&self, material: CertificateMaterial) {
        self.materials
            .write()
            .insert(material.record.domain.clone(), material);
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

#[derive(Debug)]
struct CountingResolver {
    tunnel_id: String,
    calls: AtomicUsize,
}

impl HttpHostResolver for CountingResolver {
    fn resolve_host(&self, _host: &str) -> Result<String, String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.tunnel_id.clone())
    }
}

fn https_runtime(
    provider: Arc<dyn HttpsCertificateProvider>,
    resolver: Option<Arc<dyn HttpHostResolver>>,
    routes: Vec<HttpRouteConfig>,
) -> anyhow::Result<HttpsTunnelRuntime> {
    let mut builder = HttpsTunnelRuntime::builder()
        .http_config(
            HttpRuntimeConfig::builder()
                .name("https-runtime-test")
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
        )
        .tls_config(HttpsTlsConfig::default())
        .certificate_provider(provider);
    if let Some(resolver) = resolver {
        builder = builder.host_resolver(resolver);
    }
    Ok(builder.build()?)
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

async fn tls_client(
    addr: SocketAddr,
    server_name: &str,
    material: &CertificateMaterial,
) -> anyhow::Result<TlsStream<TcpStream>> {
    let mut roots = RootCertStore::empty();
    roots.add(first_certificate_der(&material.certificate_pem)?)?;
    let provider = ring::default_provider();
    let config = ClientConfig::builder_with_provider(Arc::new(provider))
        .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])?
        .with_root_certificates(roots)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    let stream = TcpStream::connect(addr).await?;
    let server_name = ServerName::try_from(server_name.to_string())?;
    Ok(connector.connect(server_name, stream).await?)
}

fn first_certificate_der(pem: &str) -> anyhow::Result<CertificateDer<'static>> {
    let mut reader = Cursor::new(pem.as_bytes());
    let certificate = rustls_pemfile::certs(&mut reader)
        .next()
        .transpose()?
        .ok_or_else(|| anyhow::anyhow!("missing generated certificate"))?;
    Ok(certificate)
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

async fn read_response<S>(stream: &mut S) -> anyhow::Result<Response>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let head = read_head_from_stream(stream).await?;
    let text = String::from_utf8(head)?;
    let status = text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or_else(|| anyhow::anyhow!("missing status"))?
        .parse::<u16>()?;
    let content_length = header_value(&text, "Content-Length").and_then(|value| value.parse().ok());
    let body = if let Some(length) = content_length {
        let mut body = vec![0_u8; length];
        stream.read_exact(&mut body).await?;
        body
    } else {
        Vec::new()
    };

    Ok(Response { status, body })
}

async fn read_head_from_stream<S>(stream: &mut S) -> anyhow::Result<Vec<u8>>
where
    S: AsyncRead + Unpin,
{
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

fn header_value(text: &str, name: &str) -> Option<String> {
    text.lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(header, _)| header.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.trim().to_string())
}

fn assert_tls_log(metrics: &HttpsRuntimeMetrics, sni: &str) {
    let log = metrics
        .http
        .recent_requests
        .first()
        .and_then(|request| request.tls.as_ref())
        .expect("request TLS log");
    assert_eq!(log.sni.as_deref(), Some(sni));
    assert!(log.tls_version.is_some());
    assert!(log.cipher_suite.is_some());
    assert_eq!(log.certificate_domain.as_deref(), Some(sni));
}

fn assert_certificate_fingerprint(metrics: &HttpsRuntimeMetrics, fingerprint: &str) {
    assert!(
        metrics
            .certificates
            .iter()
            .any(|certificate| certificate.fingerprint_sha256 == fingerprint),
        "expected certificate fingerprint {fingerprint}"
    );
}
