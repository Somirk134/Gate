use gate_protocol::Message;

pub struct CommunicationLogger;

impl CommunicationLogger {
    pub fn send(message: &Message) {
        tracing::debug!(
            command = %message.header.command,
            request_id = %message.header.request_id,
            "communication send"
        );
    }

    pub fn receive(message: &Message) {
        tracing::debug!(
            command = %message.header.command,
            request_id = %message.header.request_id,
            "communication receive"
        );
    }
}
