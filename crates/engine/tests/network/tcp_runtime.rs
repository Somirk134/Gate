use gate_engine::runtime::{RuntimeConfig, TimeoutConfig, TunnelRuntime};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::test]
async fn tcp_runtime_forwards_bidirectional_bytes() -> anyhow::Result<()> {
    let target_listener = TcpListener::bind("127.0.0.1:0").await?;
    let target_addr = target_listener.local_addr()?;
    let target_task = tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = target_listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let (mut reader, mut writer) = stream.into_split();
                let _ = tokio::io::copy(&mut reader, &mut writer).await;
            });
        }
    });

    let runtime = TunnelRuntime::new(
        RuntimeConfig::builder()
            .name("tcp-runtime-test")
            .listen_addr("127.0.0.1:0".parse()?)
            .target_addr(target_addr)
            .timeout(
                TimeoutConfig::builder()
                    .connect_timeout(Duration::from_secs(1))
                    .idle_timeout(Duration::from_secs(10))
                    .shutdown_timeout(Duration::from_secs(2))
                    .build(),
            )
            .build(),
    );

    runtime.start().await?;
    let listen_addr = runtime.bound_addr().expect("runtime listener address");

    let mut client = TcpStream::connect(listen_addr).await?;
    client.write_all(b"gate-runtime").await?;

    let mut echoed = vec![0; "gate-runtime".len()];
    client.read_exact(&mut echoed).await?;
    assert_eq!(echoed, b"gate-runtime");

    drop(client);
    tokio::time::sleep(Duration::from_millis(100)).await;

    let metrics = runtime.metrics();
    assert!(metrics.upload >= "gate-runtime".len() as u64);
    assert!(metrics.download >= "gate-runtime".len() as u64);

    runtime.shutdown().await?;
    target_task.abort();
    Ok(())
}
