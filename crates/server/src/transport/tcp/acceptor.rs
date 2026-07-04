use tokio::net::TcpListener;

pub async fn start_tcp_acceptor(addr: &str) {
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::info!("TCP acceptor listening on {}", addr);

    loop {
        let (stream, peer) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            todo!("handle incoming TCP connection from {}", peer)
        });
    }
}
