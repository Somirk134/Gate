# Transport

Transport is the async IO boundary. The framework depends on this trait and does not depend on any concrete socket implementation.

## Rust Trait

```rust
pub trait Transport: Send + Sync {
    fn name(&self) -> &'static str;
    fn kind(&self) -> TransportKind;
    fn state(&self) -> TransportState;
    fn capabilities(&self) -> TransportCapabilities;
    fn connect<'a>(&'a self, endpoint: TransportEndpoint) -> CommunicationFuture<'a, ()>;
    fn disconnect<'a>(&'a self) -> CommunicationFuture<'a, ()>;
    fn send<'a>(&'a self, message: Message) -> CommunicationFuture<'a, ()>;
    fn receive<'a>(&'a self) -> CommunicationFuture<'a, Option<Message>>;
    fn reconnect<'a>(&'a self) -> CommunicationFuture<'a, ()>;
}
```

## Future Adapters

- Tokio TCP.
- WebSocket.
- QUIC.
- Custom plugin transports.

## Current Adapter

`MockTransport` simulates connect, send, receive, disconnect, and reconnect. It is intended for framework tests and UI integration before real network IO is attached.
