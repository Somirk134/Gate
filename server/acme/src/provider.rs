use crate::config::ChallengeType;
use crate::error::AcmeError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait AcmeProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn directory_url(&self) -> &str;

    async fn create_account(&self, contact: AcmeAccountContact) -> Result<AcmeAccount, AcmeError>;

    async fn start_order(&self, request: AcmeCertificateRequest) -> Result<AcmeOrder, AcmeError>;

    async fn prepare_challenge(
        &self,
        order_id: &str,
        challenge_type: ChallengeType,
    ) -> Result<AcmeChallenge, AcmeError>;

    async fn finalize_order(&self, order_id: &str, csr_der: &[u8]) -> Result<AcmeOrder, AcmeError>;

    async fn download_certificate(&self, order_id: &str) -> Result<String, AcmeError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetsEncryptProvider {
    directory_url: String,
}

impl Default for LetsEncryptProvider {
    fn default() -> Self {
        Self {
            directory_url: "https://acme-v02.api.letsencrypt.org/directory".to_string(),
        }
    }
}

impl LetsEncryptProvider {
    pub fn staging() -> Self {
        Self {
            directory_url: "https://acme-staging-v02.api.letsencrypt.org/directory".to_string(),
        }
    }
}

#[async_trait]
impl AcmeProvider for LetsEncryptProvider {
    fn name(&self) -> &'static str {
        "letsencrypt"
    }

    fn directory_url(&self) -> &str {
        &self.directory_url
    }

    async fn create_account(&self, _contact: AcmeAccountContact) -> Result<AcmeAccount, AcmeError> {
        Err(AcmeError::NetworkDisabled)
    }

    async fn start_order(&self, _request: AcmeCertificateRequest) -> Result<AcmeOrder, AcmeError> {
        Err(AcmeError::NetworkDisabled)
    }

    async fn prepare_challenge(
        &self,
        _order_id: &str,
        _challenge_type: ChallengeType,
    ) -> Result<AcmeChallenge, AcmeError> {
        Err(AcmeError::NetworkDisabled)
    }

    async fn finalize_order(
        &self,
        _order_id: &str,
        _csr_der: &[u8],
    ) -> Result<AcmeOrder, AcmeError> {
        Err(AcmeError::NetworkDisabled)
    }

    async fn download_certificate(&self, _order_id: &str) -> Result<String, AcmeError> {
        Err(AcmeError::NetworkDisabled)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcmeAccountContact {
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcmeAccount {
    pub id: String,
    pub contact: AcmeAccountContact,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcmeCertificateRequest {
    pub domain: String,
    pub san: Vec<String>,
    pub challenge_type: ChallengeType,
    pub preferred_chain: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcmeOrder {
    pub id: String,
    pub domain: String,
    pub status: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcmeChallenge {
    pub order_id: String,
    pub challenge_type: ChallengeType,
    pub token: String,
    pub status: String,
}
