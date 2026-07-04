use crate::domain::models::client::Client;
use anyhow::Result;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait ClientRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Client>>;
    async fn find_by_token(&self, token: &str) -> Result<Option<Client>>;
    async fn save(&self, client: &Client) -> Result<()>;
    async fn update_status(&self, id: Uuid, status: &str) -> Result<()>;
    async fn delete(&self, id: Uuid) -> Result<()>;
    async fn list_all(&self) -> Result<Vec<Client>>;
}
