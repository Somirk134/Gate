//! Handler traits and empty command handlers reserved for future business logic.

use serde::{Deserialize, Serialize};

use crate::{
    connection::ConnectionId,
    event::CommunicationEvent,
    request::{Request, Response},
    session::SessionId,
    shared::CommunicationFuture,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HandlerContext {
    pub connection_id: Option<ConnectionId>,
    pub session_id: Option<SessionId>,
}

pub trait RequestHandler: Send + Sync {
    fn name(&self) -> &'static str;

    fn handle_request<'a>(
        &'a self,
        request: Request,
        context: HandlerContext,
    ) -> CommunicationFuture<'a, Response>;
}

pub trait ResponseHandler: Send + Sync {
    fn name(&self) -> &'static str;

    fn handle_response<'a>(
        &'a self,
        response: Response,
        context: HandlerContext,
    ) -> CommunicationFuture<'a, ()>;
}

pub trait EventHandler: Send + Sync {
    fn name(&self) -> &'static str;

    fn handle_event<'a>(
        &'a self,
        event: CommunicationEvent,
        context: HandlerContext,
    ) -> CommunicationFuture<'a, ()>;
}

#[derive(Debug, Default)]
pub struct NoopRequestHandler;

impl RequestHandler for NoopRequestHandler {
    fn name(&self) -> &'static str {
        "noop"
    }

    fn handle_request<'a>(
        &'a self,
        request: Request,
        _context: HandlerContext,
    ) -> CommunicationFuture<'a, Response> {
        Box::pin(async move { Ok(Response::empty(request.id, request.command())) })
    }
}

#[derive(Debug, Default)]
pub struct NoopResponseHandler;

impl ResponseHandler for NoopResponseHandler {
    fn name(&self) -> &'static str {
        "noop_response"
    }

    fn handle_response<'a>(
        &'a self,
        _response: Response,
        _context: HandlerContext,
    ) -> CommunicationFuture<'a, ()> {
        Box::pin(async move { Ok(()) })
    }
}

#[derive(Debug, Default)]
pub struct NoopEventHandler;

impl EventHandler for NoopEventHandler {
    fn name(&self) -> &'static str {
        "noop_event"
    }

    fn handle_event<'a>(
        &'a self,
        _event: CommunicationEvent,
        _context: HandlerContext,
    ) -> CommunicationFuture<'a, ()> {
        Box::pin(async move { Ok(()) })
    }
}

macro_rules! empty_request_handler {
    ($name:ident, $label:literal) => {
        #[derive(Debug, Default)]
        pub struct $name;

        impl RequestHandler for $name {
            fn name(&self) -> &'static str {
                $label
            }

            fn handle_request<'a>(
                &'a self,
                request: Request,
                _context: HandlerContext,
            ) -> CommunicationFuture<'a, Response> {
                Box::pin(async move { Ok(Response::empty(request.id, request.command())) })
            }
        }
    };
}

empty_request_handler!(ProjectHandler, "project");
empty_request_handler!(TunnelHandler, "tunnel");
empty_request_handler!(ServerHandler, "server");
empty_request_handler!(SystemHandler, "system");
empty_request_handler!(LogHandler, "log");
