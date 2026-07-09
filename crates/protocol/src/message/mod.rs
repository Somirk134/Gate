mod heartbeat;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

pub use heartbeat::{
    HeartbeatConfig, HeartbeatInterval, HeartbeatTimeout, Ping, Pong, ReconnectInterval,
};

use crate::{command::Command, version::ProtocolVersion};

/// Top-level protocol message. Naked payloads are intentionally impossible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub header: Header,
    pub body: Body,
    pub metadata: Metadata,
}

impl Message {
    pub fn new(
        message_type: MessageType,
        command: Command,
        body: Body,
        metadata: Metadata,
    ) -> Self {
        let body_length = body.encoded_len_hint();
        Self {
            header: Header::new(message_type, command, body_length),
            body,
            metadata,
        }
    }

    pub fn request(command: Command, body: Body, metadata: Metadata) -> Self {
        Self::new(MessageType::Request, command, body, metadata)
    }

    pub fn heartbeat_ping(metadata: Metadata) -> Self {
        Self::new(
            MessageType::Heartbeat,
            Command::HeartbeatPing,
            Body::Json(serde_json::json!(Ping::default())),
            metadata,
        )
    }

    pub fn heartbeat_pong(metadata: Metadata) -> Self {
        Self::new(
            MessageType::Heartbeat,
            Command::HeartbeatPong,
            Body::Json(serde_json::json!(Pong::default())),
            metadata,
        )
    }
}

/// Message header carried by every protocol message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    pub protocol_version: ProtocolVersion,
    pub message_type: MessageType,
    pub command: Command,
    pub request_id: Uuid,
    pub trace_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub compression: Compression,
    pub encryption: Encryption,
    pub sequence: u64,
    pub body_length: u64,
    pub reserved: BTreeMap<String, String>,
}

impl Header {
    pub fn new(message_type: MessageType, command: Command, body_length: u64) -> Self {
        Self {
            protocol_version: ProtocolVersion::V1,
            message_type,
            command,
            request_id: Uuid::new_v4(),
            trace_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            compression: Compression::None,
            encryption: Encryption::None,
            sequence: 0,
            body_length,
            reserved: BTreeMap::new(),
        }
    }
}

/// Supported body envelope. Binary and plugin payloads are reserved wire shapes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", content = "value")]
pub enum Body {
    Json(Value),
    Binary(Vec<u8>),
    PluginPayload {
        plugin_id: String,
        content_type: Option<String>,
        payload: Vec<u8>,
    },
    #[default]
    Empty,
}

impl Body {
    pub fn encoded_len_hint(&self) -> u64 {
        match self {
            Self::Json(value) => serde_json::to_vec(value)
                .map(|bytes| bytes.len())
                .unwrap_or(0) as u64,
            Self::Binary(bytes) => bytes.len() as u64,
            Self::PluginPayload { payload, .. } => payload.len() as u64,
            Self::Empty => 0,
        }
    }
}

/// Metadata supplied by clients and enriched by servers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Metadata {
    pub client_version: Option<String>,
    pub platform: Option<String>,
    pub os: Option<String>,
    pub language: Option<String>,
    pub architecture: Option<String>,
    pub extra: BTreeMap<String, String>,
}

/// Wire-level message categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Event,
    Heartbeat,
    Notification,
    Broadcast,
    Error,
    Ack,
    Plugin,
}

/// Compression marker. Algorithms are negotiated outside V1 payload logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Compression {
    #[default]
    None,
    Gzip,
    Zstd,
}

/// Encryption marker. Actual TLS/AES setup belongs to transport/security layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Encryption {
    #[default]
    None,
    Tls,
    Aes,
}
