//! Encoding, decoding, and interception pipeline.

use crate::error::ProtocolError;
use bytes::Bytes;
use futures::future::BoxFuture;

/// Decodes inbound bytes into protocol frames.
pub trait Decoder: Send + Sync {
    fn decode(&self, input: Bytes) -> BoxFuture<'static, Result<Bytes, ProtocolError>>;
}

/// Encodes protocol frames into outbound bytes.
pub trait Encoder: Send + Sync {
    fn encode(&self, input: Bytes) -> BoxFuture<'static, Result<Bytes, ProtocolError>>;
}

/// Intercepts traffic for compression, encryption, audit, and metrics stages.
pub trait Interceptor: Send + Sync {
    fn intercept(&self, input: Bytes) -> BoxFuture<'static, Result<Bytes, ProtocolError>>;
}

/// Ordered protocol pipeline.
#[derive(Default)]
pub struct Pipeline {
    decoders: Vec<Box<dyn Decoder>>,
    encoders: Vec<Box<dyn Encoder>>,
    interceptors: Vec<Box<dyn Interceptor>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_decoder(&mut self, decoder: Box<dyn Decoder>) {
        self.decoders.push(decoder);
    }

    pub fn add_encoder(&mut self, encoder: Box<dyn Encoder>) {
        self.encoders.push(encoder);
    }

    pub fn add_interceptor(&mut self, interceptor: Box<dyn Interceptor>) {
        self.interceptors.push(interceptor);
    }

    pub fn decoder_count(&self) -> usize {
        self.decoders.len()
    }

    pub fn encoder_count(&self) -> usize {
        self.encoders.len()
    }

    pub fn interceptor_count(&self) -> usize {
        self.interceptors.len()
    }
}
