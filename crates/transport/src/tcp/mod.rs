use crate::Transport;

pub trait TcpTransport: Transport {}

pub trait TcpEndpoint: Send + Sync {}
