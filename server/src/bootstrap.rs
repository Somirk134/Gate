use gate_shared::error::AppError;
use gate_shared::lifecycle::ServerState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerBootstrap {
    initial_state: ServerState,
}

impl Default for ServerBootstrap {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerBootstrap {
    pub fn new() -> Self {
        Self {
            initial_state: ServerState::Starting,
        }
    }

    pub fn initial_state(&self) -> ServerState {
        self.initial_state
    }

    pub async fn boot(self) -> Result<(), AppError> {
        Ok(())
    }
}
