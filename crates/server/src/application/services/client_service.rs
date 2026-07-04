use anyhow::Result;

pub struct ClientService;

impl ClientService {
    pub fn new() -> Self {
        Self
    }

    pub async fn register(&self) -> Result<()> {
        todo!("register a new client")
    }

    pub async fn get_client(&self, client_id: &str) -> Result<()> {
        todo!("get client details")
    }
}
