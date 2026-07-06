//! Buffer pool boundary for forwarding pipelines.

use bytes::BytesMut;
use parking_lot::Mutex;

use crate::runtime::config::BufferConfig;
use crate::runtime::error::BufferError;

/// Owned buffer returned by [`BufferPool`].
#[derive(Debug)]
pub struct Buffer {
    bytes: BytesMut,
}

impl Buffer {
    pub fn new(bytes: BytesMut) -> Self {
        Self { bytes }
    }

    pub fn bytes(&self) -> &BytesMut {
        &self.bytes
    }

    pub fn bytes_mut(&mut self) -> &mut BytesMut {
        &mut self.bytes
    }

    pub fn into_inner(self) -> BytesMut {
        self.bytes
    }
}

/// Reusable fixed-size and dynamic buffer pool.
#[derive(Debug)]
pub struct BufferPool {
    config: BufferConfig,
    fixed: Mutex<Vec<BytesMut>>,
}

impl BufferPool {
    pub fn new(config: BufferConfig) -> Self {
        Self {
            config,
            fixed: Mutex::new(Vec::new()),
        }
    }

    pub fn acquire_fixed(&self) -> Buffer {
        let mut pool = self.fixed.lock();
        match pool.pop() {
            Some(mut buffer) => {
                buffer.clear();
                Buffer::new(buffer)
            }
            None => Buffer::new(BytesMut::with_capacity(self.config.fixed_buffer_size)),
        }
    }

    pub fn acquire_dynamic(&self, size: usize) -> Result<Buffer, BufferError> {
        if size > self.config.dynamic_buffer_limit {
            return Err(BufferError::SizeLimitExceeded {
                requested: size,
                limit: self.config.dynamic_buffer_limit,
            });
        }

        if size <= self.config.fixed_buffer_size {
            return Ok(self.acquire_fixed());
        }

        Ok(Buffer::new(BytesMut::with_capacity(size)))
    }

    pub fn release(&self, buffer: Buffer) {
        let mut bytes = buffer.into_inner();
        if bytes.capacity() != self.config.fixed_buffer_size {
            return;
        }

        let mut pool = self.fixed.lock();
        if pool.len() >= self.config.pool_capacity {
            return;
        }

        bytes.clear();
        pool.push(bytes);
    }

    pub fn pooled_count(&self) -> usize {
        self.fixed.lock().len()
    }

    pub fn config(&self) -> &BufferConfig {
        &self.config
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new(BufferConfig::default())
    }
}
