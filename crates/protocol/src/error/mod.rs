use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::version::ProtocolVersion;

/// Stable protocol error codes allocated for wire-level reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCode {
    Success,
    InvalidPacket,
    InvalidHeader,
    InvalidBody,
    AuthFailed,
    TunnelError,
    InternalError,
    UnknownError,
    Extension(u16),
}

impl ErrorCode {
    pub fn as_u16(self) -> u16 {
        match self {
            Self::Success => 1000,
            Self::InvalidPacket => 2000,
            Self::InvalidHeader => 2001,
            Self::InvalidBody => 2002,
            Self::AuthFailed => 3000,
            Self::TunnelError => 4000,
            Self::InternalError => 5000,
            Self::UnknownError => 6000,
            Self::Extension(code) => code,
        }
    }
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("codec error: {0}")]
    Codec(#[from] CodecError),
    #[error("serialization error: {0}")]
    Serialization(#[from] SerializeError),
    #[error("packet error: {0}")]
    Packet(#[from] PacketError),
    #[error("version error: {0}")]
    Version(#[from] VersionError),
    #[error("transport error: {0}")]
    Transport(#[from] TransportError),
    #[error("{code:?}: {message}")]
    ErrorCode { code: ErrorCode, message: String },
}

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("unsupported codec: {0}")]
    UnsupportedCodec(&'static str),
    #[error("encode failed: {0}")]
    Encode(String),
    #[error("decode failed: {0}")]
    Decode(String),
    #[error("serializer error: {0}")]
    Serializer(#[from] SerializeError),
}

#[derive(Debug, Error)]
pub enum SerializeError {
    #[error("serialize failed: {0}")]
    Serialize(String),
    #[error("deserialize failed: {0}")]
    Deserialize(String),
}

#[derive(Debug, Error)]
pub enum PacketError {
    #[error("invalid packet: {0}")]
    InvalidPacket(String),
    #[error("invalid header: {0}")]
    InvalidHeader(String),
    #[error("invalid body: {0}")]
    InvalidBody(String),
    #[error("packet too large: {actual} bytes exceeds {max} bytes")]
    TooLarge { actual: usize, max: usize },
    #[error("packet is empty")]
    Empty,
}

#[derive(Debug, Error)]
pub enum VersionError {
    #[error("unsupported protocol version: {requested}")]
    UnsupportedVersion { requested: ProtocolVersion },
    #[error("version negotiation failed: {0}")]
    NegotiationFailed(String),
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("transport unavailable: {0}")]
    Unavailable(String),
    #[error("transport read failed: {0}")]
    Read(String),
    #[error("transport write failed: {0}")]
    Write(String),
    #[error("transport closed")]
    Closed,
}
