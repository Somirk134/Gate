use crate::domain::models::tunnel::Tunnel;
use anyhow::Result;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait TunnelRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Tunnel>>;
    async fn find_by_client_id(&self, client_id: Uuid) -> Result<Vec<Tunnel>>;
    async fn save(&self, tunnel: &Tunnel) -> Result<()>;
    async fn update_status(&self, id: Uuid, status: &str) -> Result<()>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn list_active(&self) -> Result<Vec<Tunnel>>;
}
