//! Stream wrappers with statistics hooks.

use crate::runtime::monitor::TrafficStatistics;
use crate::runtime::session::SessionId;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// Runtime stream role used to map bytes to upload or download direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StreamRole {
    Client,
    Target,
}

/// Stream metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamContext {
    pub session_id: SessionId,
    pub role: StreamRole,
    pub created_at_millis: u64,
}

impl StreamContext {
    pub fn new(session_id: SessionId, role: StreamRole) -> Self {
        Self {
            session_id,
            role,
            created_at_millis: now_millis(),
        }
    }
}

/// Atomic per-stream counters.
#[derive(Debug)]
pub struct StreamStatistics {
    bytes_read: AtomicU64,
    bytes_written: AtomicU64,
    read_ops: AtomicU64,
    write_ops: AtomicU64,
    error_count: AtomicU64,
    last_activity_millis: AtomicU64,
}

impl Default for StreamStatistics {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamStatistics {
    pub fn new() -> Self {
        Self {
            bytes_read: AtomicU64::new(0),
            bytes_written: AtomicU64::new(0),
            read_ops: AtomicU64::new(0),
            write_ops: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            last_activity_millis: AtomicU64::new(now_millis()),
        }
    }

    pub fn record_read(&self, bytes: u64) {
        if bytes == 0 {
            return;
        }
        self.bytes_read.fetch_add(bytes, Ordering::Relaxed);
        self.read_ops.fetch_add(1, Ordering::Relaxed);
        self.touch();
    }

    pub fn record_write(&self, bytes: u64) {
        if bytes == 0 {
            return;
        }
        self.bytes_written.fetch_add(bytes, Ordering::Relaxed);
        self.write_ops.fetch_add(1, Ordering::Relaxed);
        self.touch();
    }

    pub fn record_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
        self.touch();
    }

    pub fn bytes_read(&self) -> u64 {
        self.bytes_read.load(Ordering::Relaxed)
    }

    pub fn bytes_written(&self) -> u64 {
        self.bytes_written.load(Ordering::Relaxed)
    }

    pub fn last_activity_millis(&self) -> u64 {
        self.last_activity_millis.load(Ordering::Relaxed)
    }

    fn touch(&self) {
        self.last_activity_millis
            .store(now_millis(), Ordering::Relaxed);
    }
}

/// AsyncRead wrapper.
#[derive(Debug)]
pub struct StreamReader<R> {
    inner: R,
    statistics: Arc<StreamStatistics>,
}

impl<R> StreamReader<R> {
    pub fn new(inner: R, statistics: Arc<StreamStatistics>) -> Self {
        Self { inner, statistics }
    }

    pub fn statistics(&self) -> Arc<StreamStatistics> {
        Arc::clone(&self.statistics)
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R> AsyncRead for StreamReader<R>
where
    R: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let before = buf.filled().len();
        let poll = Pin::new(&mut self.inner).poll_read(cx, buf);
        if let Poll::Ready(result) = &poll {
            match result {
                Ok(()) => {
                    let bytes = buf.filled().len().saturating_sub(before) as u64;
                    self.statistics.record_read(bytes);
                }
                Err(_) => self.statistics.record_error(),
            }
        }
        poll
    }
}

/// AsyncWrite wrapper.
#[derive(Debug)]
pub struct StreamWriter<W> {
    inner: W,
    statistics: Arc<StreamStatistics>,
}

impl<W> StreamWriter<W> {
    pub fn new(inner: W, statistics: Arc<StreamStatistics>) -> Self {
        Self { inner, statistics }
    }

    pub fn statistics(&self) -> Arc<StreamStatistics> {
        Arc::clone(&self.statistics)
    }

    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W> AsyncWrite for StreamWriter<W>
where
    W: AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let poll = Pin::new(&mut self.inner).poll_write(cx, buf);
        if let Poll::Ready(result) = &poll {
            match result {
                Ok(bytes) => self.statistics.record_write(*bytes as u64),
                Err(_) => self.statistics.record_error(),
            }
        }
        poll
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

/// Full-duplex stream wrapper used by `copy_bidirectional`.
#[derive(Debug)]
pub struct InstrumentedStream<S> {
    inner: S,
    role: StreamRole,
    statistics: Arc<StreamStatistics>,
    runtime_traffic: Arc<TrafficStatistics>,
    session_traffic: Arc<TrafficStatistics>,
}

impl<S> InstrumentedStream<S> {
    pub fn new(
        inner: S,
        role: StreamRole,
        statistics: Arc<StreamStatistics>,
        runtime_traffic: Arc<TrafficStatistics>,
        session_traffic: Arc<TrafficStatistics>,
    ) -> Self {
        Self {
            inner,
            role,
            statistics,
            runtime_traffic,
            session_traffic,
        }
    }

    pub fn statistics(&self) -> Arc<StreamStatistics> {
        Arc::clone(&self.statistics)
    }

    pub fn into_inner(self) -> S {
        self.inner
    }

    fn record_directional_read(&self, bytes: u64) {
        match self.role {
            StreamRole::Client => {
                self.runtime_traffic.record_upload(bytes);
                self.session_traffic.record_upload(bytes);
            }
            StreamRole::Target => {
                self.runtime_traffic.record_download(bytes);
                self.session_traffic.record_download(bytes);
            }
        }
    }
}

impl<S> AsyncRead for InstrumentedStream<S>
where
    S: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let before = buf.filled().len();
        let poll = Pin::new(&mut self.inner).poll_read(cx, buf);
        if let Poll::Ready(result) = &poll {
            match result {
                Ok(()) => {
                    let bytes = buf.filled().len().saturating_sub(before) as u64;
                    self.statistics.record_read(bytes);
                    self.record_directional_read(bytes);
                }
                Err(_) => {
                    self.statistics.record_error();
                    self.runtime_traffic.increment_error();
                    self.session_traffic.increment_error();
                }
            }
        }
        poll
    }
}

impl<S> AsyncWrite for InstrumentedStream<S>
where
    S: AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let poll = Pin::new(&mut self.inner).poll_write(cx, buf);
        if let Poll::Ready(result) = &poll {
            match result {
                Ok(bytes) => self.statistics.record_write(*bytes as u64),
                Err(_) => {
                    self.statistics.record_error();
                    self.runtime_traffic.increment_error();
                    self.session_traffic.increment_error();
                }
            }
        }
        poll
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
