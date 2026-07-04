use anyhow::Result;

pub struct AuthenticateClientUseCase;

impl AuthenticateClientUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, client_id: &str, secret: &str) -> Result<String> {
        todo!("execute client authentication and return token")
    }
}
