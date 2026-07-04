use anyhow::Result;

pub struct AuthService;

impl AuthService {
    pub fn new() -> Self {
        Self
    }

    pub async fn authenticate(&self, token: &str) -> Result<bool> {
        todo!("authenticate client token")
    }

    pub async fn generate_token(&self, client_id: &str) -> Result<String> {
        todo!("generate JWT token for client")
    }
}
