use super::{
    CertificateAlgorithm, CertificateFingerprint, CertificateRecord, CertificateStatus,
    StoredCertificate,
};
use crate::config::ChallengeType;
use crate::error::CertificateError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait CertificateManager: Send + Sync {
    async fn request_certificate(
        &self,
        request: CertificateRequest,
    ) -> Result<CertificateRecord, CertificateError>;

    async fn load_certificate(&self, domain: &str) -> Result<StoredCertificate, CertificateError>;

    async fn query_certificate(
        &self,
        query: CertificateQuery,
    ) -> Result<Option<CertificateRecord>, CertificateError>;

    async fn update_certificate(
        &self,
        domain: &str,
        update: CertificateUpdate,
    ) -> Result<CertificateRecord, CertificateError>;

    async fn delete_certificate(&self, domain: &str) -> Result<(), CertificateError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateRequest {
    pub domain: String,
    pub san: Vec<String>,
    pub issuer: String,
    pub algorithm: CertificateAlgorithm,
    pub challenge_type: ChallengeType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateQuery {
    pub domain: String,
    pub include_expired: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateUpdate {
    pub expire_time: Option<DateTime<Utc>>,
    pub renew_time: Option<DateTime<Utc>>,
    pub status: Option<CertificateStatus>,
    pub fingerprint: Option<CertificateFingerprint>,
    pub algorithm: Option<CertificateAlgorithm>,
    pub san: Option<Vec<String>>,
    pub certificate_pem: Option<String>,
    pub private_key_pem: Option<String>,
}
