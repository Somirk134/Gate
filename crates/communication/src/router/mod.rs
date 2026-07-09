//! Message router and command-to-handler mapping.

use dashmap::DashMap;
use gate_protocol::Command;
use std::sync::Arc;

use crate::{
    handler::{
        HandlerContext, LogHandler, NoopRequestHandler, ProjectHandler, RequestHandler,
        ServerHandler, SystemHandler, TunnelHandler,
    },
    request::{Request, Response},
    shared::CommunicationFuture,
};

#[derive(Default)]
pub struct MessageRouter {
    handlers: DashMap<Command, Arc<dyn RequestHandler>>,
    fallback: Arc<NoopRequestHandler>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            handlers: DashMap::new(),
            fallback: Arc::new(NoopRequestHandler),
        }
    }

    pub fn with_default_handlers() -> Self {
        let router = Self::new();
        router.register_default_handlers();
        router
    }

    pub fn register(&self, command: Command, handler: Arc<dyn RequestHandler>) {
        self.handlers.insert(command, handler);
    }

    pub fn unregister(&self, command: &Command) -> Option<Arc<dyn RequestHandler>> {
        self.handlers.remove(command).map(|(_, handler)| handler)
    }

    pub fn route<'a>(
        &'a self,
        request: Request,
        context: HandlerContext,
    ) -> CommunicationFuture<'a, Response> {
        let handler = self
            .handlers
            .get(&request.command())
            .map(|entry| Arc::clone(entry.value()))
            .unwrap_or_else(|| {
                let handler: Arc<dyn RequestHandler> = self.fallback.clone();
                handler
            });

        Box::pin(async move { handler.handle_request(request, context).await })
    }

    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    fn register_default_handlers(&self) {
        let project = Arc::new(ProjectHandler);
        self.register(Command::ProjectCreate, project.clone());
        self.register(Command::ProjectUpdate, project.clone());
        self.register(Command::ProjectDelete, project.clone());
        self.register(Command::ProjectList, project);

        let tunnel = Arc::new(TunnelHandler);
        self.register(Command::TunnelCreate, tunnel.clone());
        self.register(Command::TunnelRegister, tunnel.clone());
        self.register(Command::TunnelStart, tunnel.clone());
        self.register(Command::TunnelStop, tunnel.clone());
        self.register(Command::TunnelRestart, tunnel.clone());
        self.register(Command::TunnelStatistics, tunnel.clone());
        self.register(Command::TunnelRelayConnect, tunnel.clone());
        self.register(Command::TunnelRelayStart, tunnel);

        let server = Arc::new(ServerHandler);
        self.register(Command::ServerConnect, server.clone());
        self.register(Command::ServerDisconnect, server.clone());
        self.register(Command::ServerStatus, server);

        let log = Arc::new(LogHandler);
        self.register(Command::LogSubscribe, log.clone());
        self.register(Command::LogUnsubscribe, log.clone());
        self.register(Command::LogQuery, log);

        let system = Arc::new(SystemHandler);
        self.register(Command::StatisticsQuery, system.clone());
        self.register(Command::SettingsGet, system.clone());
        self.register(Command::SettingsSet, system.clone());
        self.register(Command::HeartbeatPing, system.clone());
        self.register(Command::HeartbeatPong, system.clone());
        self.register(Command::AuthLogin, system.clone());
        self.register(Command::AuthLogout, system.clone());
        self.register(Command::AuthRefresh, system.clone());
        self.register(Command::SystemVersion, system.clone());
        self.register(Command::SystemHealth, system.clone());
        self.register(Command::SystemShutdown, system);
    }
}
