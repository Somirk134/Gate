//! Incoming, outgoing, and reserved priority queues for protocol messages.

use chrono::{DateTime, Utc};
use gate_protocol::Message;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub id: Uuid,
    pub message: Message,
    pub priority: u8,
    pub enqueued_at: DateTime<Utc>,
}

impl MessageEnvelope {
    pub fn new(message: Message) -> Self {
        Self {
            id: Uuid::new_v4(),
            message,
            priority: 0,
            enqueued_at: Utc::now(),
        }
    }

    pub fn with_priority(message: Message, priority: u8) -> Self {
        Self {
            priority,
            ..Self::new(message)
        }
    }
}

#[derive(Debug, Default)]
pub struct MessageQueue {
    inner: RwLock<VecDeque<MessageEnvelope>>,
}

impl MessageQueue {
    pub fn enqueue(&self, envelope: MessageEnvelope) {
        self.inner.write().push_back(envelope);
    }

    pub fn enqueue_message(&self, message: Message) {
        self.enqueue(MessageEnvelope::new(message));
    }

    pub fn dequeue(&self) -> Option<MessageEnvelope> {
        self.inner.write().pop_front()
    }

    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }

    pub fn clear(&self) {
        self.inner.write().clear();
    }
}

pub type OutgoingQueue = MessageQueue;
pub type IncomingQueue = MessageQueue;

#[derive(Debug, Default)]
pub struct PriorityMessageQueue {
    inner: RwLock<Vec<MessageEnvelope>>,
}

impl PriorityMessageQueue {
    pub fn enqueue(&self, envelope: MessageEnvelope) {
        let mut inner = self.inner.write();
        inner.push(envelope);
        inner.sort_by(|left, right| {
            right
                .priority
                .cmp(&left.priority)
                .then_with(|| left.enqueued_at.cmp(&right.enqueued_at))
        });
    }

    pub fn dequeue(&self) -> Option<MessageEnvelope> {
        if self.inner.read().is_empty() {
            return None;
        }

        Some(self.inner.write().remove(0))
    }

    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }
}
