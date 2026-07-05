//! Command, response, and event dispatching.

use std::sync::Arc;

use gate_protocol::{Message, MessageType};

use crate::{
    error::{CommunicationResult, DispatcherError},
    handler::HandlerContext,
    request::{Request, RequestManager, Response},
    router::MessageRouter,
    shared::CommunicationFuture,
};

pub use crate::event::EventDispatcher;

pub trait Dispatcher: Send + Sync {
    fn dispatch<'a>(
        &'a self,
        message: Message,
        context: HandlerContext,
    ) -> CommunicationFuture<'a, Option<Message>>;
}

pub struct CommandDispatcher {
    router: MessageRouter,
}

impl CommandDispatcher {
    pub fn new(router: MessageRouter) -> Self {
        Self { router }
    }

    pub fn with_default_router() -> Self {
        Self {
            router: MessageRouter::with_default_handlers(),
        }
    }

    pub fn router(&self) -> &MessageRouter {
        &self.router
    }
}

impl Default for CommandDispatcher {
    fn default() -> Self {
        Self::with_default_router()
    }
}

impl Dispatcher for CommandDispatcher {
    fn dispatch<'a>(
        &'a self,
        message: Message,
        context: HandlerContext,
    ) -> CommunicationFuture<'a, Option<Message>> {
        Box::pin(async move {
            if message.header.message_type != MessageType::Request {
                return Err(DispatcherError::UnsupportedMessageType {
                    message_type: message.header.message_type,
                }
                .into());
            }

            let request = Request::new(message, 30_000);
            let response = self.router.route(request, context).await?;
            Ok(Some(response.message))
        })
    }
}

pub struct ResponseDispatcher {
    request_manager: Arc<RequestManager>,
}

impl ResponseDispatcher {
    pub fn new(request_manager: Arc<RequestManager>) -> Self {
        Self { request_manager }
    }

    pub fn dispatch_response(&self, message: Message) -> CommunicationResult<()> {
        if message.header.message_type != MessageType::Response {
            return Err(DispatcherError::UnsupportedMessageType {
                message_type: message.header.message_type,
            }
            .into());
        }

        self.request_manager.resolve(Response::from_message(message))
    }

    pub fn request_manager(&self) -> Arc<RequestManager> {
        Arc::clone(&self.request_manager)
    }
}
