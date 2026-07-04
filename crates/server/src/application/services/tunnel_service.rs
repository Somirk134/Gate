use anyhow::Result;

pub struct TunnelService;

impl TunnelService {
    pub fn new() -> Self {
        Self
    }

    pub async fn create_tunnel(&self) -> Result<()> {
        todo!("create tunnel between client and server")
    }

    pub async fn close_tunnel(&self) -> Result<()> {
        todo!("close tunnel")
    }

    pub async fn list_tunnels(&self) -> Result<Vec<String>> {
        todo!("list all active tunnels")
    }
}
