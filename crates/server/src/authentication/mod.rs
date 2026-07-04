use anyhow::Result;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub client_id: Uuid,
}

pub struct JwtAuthenticator {
    secret: String,
}

impl JwtAuthenticator {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
        }
    }

    pub fn generate_token(&self, client_id: Uuid) -> Result<String> {
        let now = chrono::Utc::now();
        let claims = Claims {
            sub: client_id.to_string(),
            exp: (now + chrono::Duration::hours(24)).timestamp() as usize,
            iat: now.timestamp() as usize,
            client_id,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )?;

        Ok(token)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }
}
