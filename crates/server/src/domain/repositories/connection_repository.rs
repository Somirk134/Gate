use crate::domain::models::connection::Connection;
use anyhow::Result;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait ConnectionRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Connection>>;
    async fn find_by_client_id(&self, client_id: Uuid) -> Result<Vec<Connection>>;
    async fn save(&self, connection: &Connection) -> Result<()>;
    async fn update_disconnect(&self, id: Uuid) -> Result<()>;
}
