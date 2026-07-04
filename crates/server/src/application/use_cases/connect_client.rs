use anyhow::Result;

pub struct ConnectClientUseCase;

impl ConnectClientUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, client_id: &str, token: &str) -> Result<()> {
        todo!("execute client connection flow")
    }
}
