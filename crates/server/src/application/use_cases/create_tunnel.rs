use anyhow::Result;

pub struct CreateTunnelUseCase;

impl CreateTunnelUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, client_id: &str, remote_port: u16, protocol: &str) -> Result<()> {
        todo!("execute tunnel creation flow")
    }
}
