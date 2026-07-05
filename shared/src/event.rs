use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventMetadata {
    pub id: Uuid,
    pub name: String,
    pub occurred_at: DateTime<Utc>,
}

pub trait Event: Send + Sync {
    fn metadata(&self) -> &EventMetadata;
}

pub trait Publisher<E>: Send + Sync
where
    E: Event,
{
    fn publish(&self, event: E) -> Result<(), AppError>;
}

pub trait Subscriber<E>: Send + Sync
where
    E: Event,
{
    fn subscribe(&self, handler: Box<dyn EventHandler<E>>) -> Result<(), AppError>;
}

pub trait Dispatcher<E>: Send + Sync
where
    E: Event,
{
    fn dispatch(&self, event: E) -> Result<(), AppError>;
}

pub trait EventHandler<E>: Send + Sync
where
    E: Event,
{
    fn handle(&self, event: &E) -> Result<(), AppError>;
}
