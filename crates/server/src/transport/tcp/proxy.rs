use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncWrite};

pub async fn proxy_copy<R, W>(mut reader: R, mut writer: W) -> std::io::Result<u64>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    tokio::io::copy(&mut reader, &mut writer).await
}
