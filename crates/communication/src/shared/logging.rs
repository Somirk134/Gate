use gate_protocol::{Command, Message};

use crate::connection::{ConnectionId, ConnectionState};

/// Thin logging adapter for communication lifecycle events.
///
/// The implementation is intentionally based on `tracing` so applications can
/// plug in JSON, terminal, file, or OpenTelemetry subscribers without changing
/// the communication layer.
pub struct CommunicationLogger;

impl CommunicationLogger {
    pub fn connection(id: ConnectionId, state: ConnectionState) {
        tracing::info!(connection_id = %id, ?state, "communication connection state changed");
    }

    pub fn send(id: Option<ConnectionId>, message: &Message) {
        let connection_id = id.map(|value| value.to_string());

        tracing::debug!(
            connection_id = connection_id.as_deref().unwrap_or(""),
            command = %message.header.command,
            request_id = %message.header.request_id,
            "communication send"
        );
    }

    pub fn receive(id: Option<ConnectionId>, message: &Message) {
        let connection_id = id.map(|value| value.to_string());

        tracing::debug!(
            connection_id = connection_id.as_deref().unwrap_or(""),
            command = %message.header.command,
            request_id = %message.header.request_id,
            "communication receive"
        );
    }

    pub fn retry(command: &Command, attempt: u32) {
        tracing::warn!(command = %command, attempt, "communication retry scheduled");
    }

    pub fn disconnect(id: Option<ConnectionId>, reason: &str) {
        let connection_id = id.map(|value| value.to_string());

        tracing::info!(
            connection_id = connection_id.as_deref().unwrap_or(""),
            reason,
            "communication disconnect"
        );
    }

    pub fn reconnect(id: Option<ConnectionId>, attempt: u32) {
        let connection_id = id.map(|value| value.to_string());

        tracing::warn!(
            connection_id = connection_id.as_deref().unwrap_or(""),
            attempt,
            "communication reconnect"
        );
    }
}
