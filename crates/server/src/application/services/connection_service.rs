use anyhow::Result;

pub struct ConnectionService;

impl ConnectionService {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle_connection(&self) -> Result<()> {
        todo!("handle new client connection")
    }

    pub async fn disconnect(&self, client_id: &str) -> Result<()> {
        todo!("handle client disconnection")
    }
}
