use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateRecord {
    pub domain: String,
    pub issuer: String,
    pub expire_time: DateTime<Utc>,
    pub create_time: DateTime<Utc>,
    pub renew_time: Option<DateTime<Utc>>,
    pub status: CertificateStatus,
    pub fingerprint: CertificateFingerprint,
    pub algorithm: CertificateAlgorithm,
    pub san: Vec<String>,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub serial_number: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredCertificate {
    pub record: CertificateRecord,
    pub certificate_pem: String,
    pub private_key_pem: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateFingerprint {
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateStatus {
    Pending,
    Active,
    ExpiringSoon,
    Expired,
    Revoked,
    Deleted,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateAlgorithm {
    Rsa,
    EcdsaP256,
    EcdsaP384,
    Ed25519,
    Unknown(String),
}

impl CertificateAlgorithm {
    pub fn is_supported_for_tls(&self) -> bool {
        matches!(
            self,
            Self::Rsa | Self::EcdsaP256 | Self::EcdsaP384 | Self::Ed25519
        )
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Rsa => "RSA",
            Self::EcdsaP256 => "ECDSA-P256",
            Self::EcdsaP384 => "ECDSA-P384",
            Self::Ed25519 => "Ed25519",
            Self::Unknown(value) => value.as_str(),
        }
    }
}
