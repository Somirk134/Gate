//! Request/response correlation and pending request management.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use gate_protocol::{Body, Command, Message, MessageType, Metadata};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::{
    error::{CommunicationError, CommunicationResult, DispatcherError, TimeoutError},
    reconnect::RetryPolicy,
    timeout::TimeoutKind,
};

pub type RequestId = Uuid;
pub type ResponseReceiver = oneshot::Receiver<Response>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequestStatus {
    Pending,
    Completed,
    TimedOut,
    Canceled,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    pub id: RequestId,
    pub message: Message,
    pub timeout_ms: u64,
    pub created_at: DateTime<Utc>,
    pub retry_policy: Option<RetryPolicy>,
}

impl Request {
    pub fn new(message: Message, timeout_ms: u64) -> Self {
        Self {
            id: message.header.request_id,
            message,
            timeout_ms,
            created_at: Utc::now(),
            retry_policy: None,
        }
    }

    pub fn from_command(command: Command, body: Body, metadata: Metadata, timeout_ms: u64) -> Self {
        Self::new(Message::request(command, body, metadata), timeout_ms)
    }

    pub fn command(&self) -> Command {
        self.message.header.command.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    pub request_id: RequestId,
    pub message: Message,
    pub latency_ms: Option<u64>,
}

impl Response {
    pub fn from_message(message: Message) -> Self {
        Self {
            request_id: message.header.request_id,
            message,
            latency_ms: None,
        }
    }

    pub fn empty(request_id: RequestId, command: Command) -> Self {
        let mut message = Message::new(
            MessageType::Response,
            command,
            Body::Empty,
            Metadata::default(),
        );
        message.header.request_id = request_id;

        Self {
            request_id,
            message,
            latency_ms: Some(0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingRequestSnapshot {
    pub id: RequestId,
    pub command: Command,
    pub status: RequestStatus,
    pub created_at: DateTime<Utc>,
    pub timeout_at: DateTime<Utc>,
}

struct PendingRequest {
    request: Request,
    timeout_at: DateTime<Utc>,
    sender: Mutex<Option<oneshot::Sender<Response>>>,
}

impl PendingRequest {
    fn snapshot(&self, status: RequestStatus) -> PendingRequestSnapshot {
        PendingRequestSnapshot {
            id: self.request.id,
            command: self.request.command(),
            status,
            created_at: self.request.created_at,
            timeout_at: self.timeout_at,
        }
    }
}

#[derive(Default)]
pub struct RequestManager {
    pending: DashMap<RequestId, PendingRequest>,
}

impl RequestManager {
    pub fn register(&self, request: Request) -> CommunicationResult<ResponseReceiver> {
        let request_id = request.id;
        let timeout_at =
            request.created_at + chrono::Duration::milliseconds(request.timeout_ms as i64);
        let (sender, receiver) = oneshot::channel();

        self.pending.insert(
            request_id,
            PendingRequest {
                request,
                timeout_at,
                sender: Mutex::new(Some(sender)),
            },
        );

        Ok(receiver)
    }

    pub fn resolve(&self, response: Response) -> CommunicationResult<()> {
        let request_id = response.request_id;
        let Some((_, pending)) = self.pending.remove(&request_id) else {
            return Err(DispatcherError::RequestNotFound { request_id }.into());
        };

        if let Some(sender) = pending.sender.lock().take() {
            sender
                .send(response)
                .map_err(|_| CommunicationError::Canceled)?;
        }

        Ok(())
    }

    pub fn cancel(&self, request_id: RequestId) -> CommunicationResult<()> {
        if self.pending.remove(&request_id).is_some() {
            return Ok(());
        }

        Err(DispatcherError::RequestNotFound { request_id }.into())
    }

    pub fn timeout(&self, request_id: RequestId) -> CommunicationResult<()> {
        let Some((_, pending)) = self.pending.remove(&request_id) else {
            return Err(DispatcherError::RequestNotFound { request_id }.into());
        };

        Err(TimeoutError::Expired {
            kind: TimeoutKind::Request,
            timeout_ms: pending.request.timeout_ms,
        }
        .into())
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    pub fn pending_snapshots(&self) -> Vec<PendingRequestSnapshot> {
        self.pending
            .iter()
            .map(|entry| entry.value().snapshot(RequestStatus::Pending))
            .collect()
    }
}
