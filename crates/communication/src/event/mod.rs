//! Event publish/subscribe dispatcher.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::CommunicationResult,
    handler::{EventHandler, HandlerContext},
};

pub type EventName = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventSubscriptionId(Uuid);

impl EventSubscriptionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EventSubscriptionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommunicationEvent {
    pub name: EventName,
    pub payload: Value,
    pub source: Option<String>,
    pub priority: i32,
    pub timestamp: DateTime<Utc>,
}

impl CommunicationEvent {
    pub fn new(name: impl Into<EventName>, payload: Value) -> Self {
        Self {
            name: name.into(),
            payload,
            source: None,
            priority: 0,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Clone)]
pub struct EventSubscription {
    pub id: EventSubscriptionId,
    pub name: EventName,
    pub priority: i32,
    pub handler: Arc<dyn EventHandler>,
}

#[derive(Default)]
pub struct EventDispatcher {
    subscriptions: DashMap<EventName, Vec<EventSubscription>>,
}

impl EventDispatcher {
    pub fn subscribe(
        &self,
        name: impl Into<EventName>,
        handler: Arc<dyn EventHandler>,
        priority: i32,
    ) -> EventSubscriptionId {
        let name = name.into();
        let subscription = EventSubscription {
            id: EventSubscriptionId::new(),
            name: name.clone(),
            priority,
            handler,
        };
        let id = subscription.id;

        let mut subscriptions = self.subscriptions.entry(name).or_default();
        subscriptions.push(subscription);
        subscriptions.sort_by(|left, right| right.priority.cmp(&left.priority));

        id
    }

    pub fn remove(&self, name: &str, id: EventSubscriptionId) -> bool {
        let Some(mut subscriptions) = self.subscriptions.get_mut(name) else {
            return false;
        };

        let before = subscriptions.len();
        subscriptions.retain(|subscription| subscription.id != id);
        before != subscriptions.len()
    }

    pub async fn publish(
        &self,
        event: CommunicationEvent,
        context: HandlerContext,
    ) -> CommunicationResult<()> {
        let subscriptions = self
            .subscriptions
            .get(&event.name)
            .map(|entry| entry.value().clone())
            .unwrap_or_default();

        for subscription in subscriptions {
            subscription
                .handler
                .handle_event(event.clone(), context.clone())
                .await?;
        }

        Ok(())
    }

    pub async fn broadcast(
        &self,
        events: Vec<CommunicationEvent>,
        context: HandlerContext,
    ) -> CommunicationResult<()> {
        for event in events {
            self.publish(event, context.clone()).await?;
        }

        Ok(())
    }

    pub fn clear(&self, name: Option<&str>) {
        if let Some(name) = name {
            self.subscriptions.remove(name);
            return;
        }

        self.subscriptions.clear();
    }

    pub fn subscriber_count(&self, name: Option<&str>) -> usize {
        if let Some(name) = name {
            return self
                .subscriptions
                .get(name)
                .map(|entry| entry.value().len())
                .unwrap_or_default();
        }

        self.subscriptions
            .iter()
            .map(|entry| entry.value().len())
            .sum()
    }
}
