use crate::domain::models::user::User;
use anyhow::Result;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn save(&self, user: &User) -> Result<()>;
    async fn delete(&self, id: Uuid) -> Result<()>;
}
