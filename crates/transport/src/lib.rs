pub mod http;
pub mod ipc;
pub mod tcp;
pub mod websocket;

pub trait Transport: Send + Sync {
    fn name(&self) -> &'static str;
}
