#![forbid(unsafe_code)]

//! Client/server communication layer for Gate.
//!
//! This crate owns the communication framework that sits above the protocol
//! layer and below future business modules. It defines transports, connection
//! lifecycle, request/response correlation, command dispatching, event
//! delivery, sessions, retry/timeout policy, metrics, and mocks.
//!
//! It intentionally does not implement tunnel forwarding, authentication,
//! heartbeat loops, business handlers, or real network sockets.

pub mod client;
pub mod connection;
pub mod dispatcher;
pub mod error;
pub mod event;
pub mod handler;
pub mod metrics;
pub mod mock;
pub mod queue;
pub mod reconnect;
pub mod request;
pub mod router;
pub mod server;
pub mod session;
pub mod shared;
pub mod timeout;
pub mod transport;

pub use client::{
    ClientConnection, ClientDispatcher, ClientEventManager, ClientRequestManager, ClientState,
    ClientTransport, CommunicationService, ConnectionPool,
};
pub use connection::{
    BasicConnection, Connection, ConnectionContext, ConnectionId, ConnectionMetadata,
    ConnectionRole, ConnectionState, ConnectionStatistics,
};
pub use dispatcher::{CommandDispatcher, Dispatcher, EventDispatcher, ResponseDispatcher};
pub use error::{
    CommunicationError, CommunicationResult, ConnectionError, DispatcherError, TimeoutError,
    TransportError,
};
pub use event::{CommunicationEvent, EventName, EventSubscription, EventSubscriptionId};
pub use handler::{
    EventHandler, HandlerContext, LogHandler, NoopEventHandler, NoopRequestHandler,
    NoopResponseHandler, ProjectHandler, RequestHandler, ResponseHandler, ServerHandler,
    SystemHandler, TunnelHandler,
};
pub use metrics::{CommunicationMetrics, MetricsSnapshot};
pub use queue::{IncomingQueue, MessageEnvelope, MessageQueue, OutgoingQueue, PriorityMessageQueue};
pub use reconnect::{RetryContext, RetryPolicy};
pub use request::{
    PendingRequestSnapshot, Request, RequestId, RequestManager, RequestStatus, Response,
    ResponseReceiver,
};
pub use router::MessageRouter;
pub use server::{
    ClientId, ClientRegistry, ConnectionManager, ConnectionRegistry, ServerConnection,
    ServerDispatcher, ServerState, ServerTransport,
};
pub use session::{
    ClientSession, ServerSession, Session, SessionContext, SessionId, SessionManager, SessionRole,
    SessionState, SessionStore,
};
pub use timeout::{TimeoutConfig, TimeoutKind};
pub use transport::{
    TcpTransport, Transport, TransportCapabilities, TransportEndpoint, TransportKind, TransportState,
};
