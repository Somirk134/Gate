#![forbid(unsafe_code)]

//! Gate 客户端在 Protocol 之上使用的 TCP 通信层。

pub mod error;
pub mod shared;
pub mod transport;

pub use error::{CommunicationError, CommunicationResult, TransportError};
pub use transport::{
    TcpTransport, Transport, TransportCapabilities, TransportEndpoint, TransportKind,
    TransportState,
};
