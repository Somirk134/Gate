//! Protocol layer for Gate.
//!
//! This crate defines the only communication contract between desktop clients
//! and Rust servers. It intentionally contains no tunnel forwarding,
//! authentication workflow, or business execution logic.

pub mod codec;
pub mod command;
pub mod error;
pub mod event;
pub mod frame;
pub mod message;
pub mod packet;
pub mod protocol;
pub mod serializer;
pub mod shared;
pub mod transport;
pub mod version;

pub use codec::{CborCodec, Codec, JsonCodec, MessagePackCodec, ProtobufCodec};
pub use command::Command;
pub use error::{
    CodecError, ErrorCode, PacketError, ProtocolError, SerializeError, TransportError,
    VersionError,
};
pub use event::ProtocolEvent;
pub use frame::{Frame, FrameCodec, FrameDecoder, FrameEncoder, FrameReader, FrameWriter};
pub use message::{
    Body, Compression, Encryption, Header, HeartbeatConfig, HeartbeatInterval,
    HeartbeatTimeout, Message, MessageType, Metadata, Ping, Pong, ReconnectInterval,
};
pub use packet::{Packet, PacketBuilder, PacketParser, PacketReader, PacketValidator, PacketWriter};
pub use protocol::{
    AuthenticationState, ConnectionState, GateProtocol, Protocol, ProtocolBuilder,
    ProtocolContext, ProtocolManager, ProtocolRegistry, ProtocolState, TunnelState,
};
pub use serializer::{JsonSerializer, Serializer};
pub use shared::{ClientInfo, ErrorInfo, ServerInfo, SystemInfo, VersionInfo};
pub use transport::{Transport, TransportEndpoint, TransportFuture};
pub use version::{ProtocolVersion, VersionNegotiation, VersionPolicy};
