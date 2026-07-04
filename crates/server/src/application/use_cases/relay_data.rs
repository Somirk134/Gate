use anyhow::Result;
use bytes::Bytes;

pub struct RelayDataUseCase;

impl RelayDataUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, tunnel_id: &str, data: Bytes) -> Result<()> {
        todo!("relay data through tunnel")
    }
}
