use crate::certificate::{CertificateRecord, StoredCertificate};
use crate::error::CertificateError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub trait CertificateStore: Clone + Send + Sync {
    fn save(&self, certificate: &StoredCertificate) -> Result<(), CertificateError>;
    fn load(&self, domain: &str) -> Result<StoredCertificate, CertificateError>;
    fn query(&self, domain: &str) -> Result<Option<CertificateRecord>, CertificateError>;
    fn list(&self) -> Result<Vec<CertificateRecord>, CertificateError>;
    fn delete(&self, domain: &str) -> Result<(), CertificateError>;
    fn contains(&self, domain: &str) -> Result<bool, CertificateError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateStoreConfig {
    pub backend: StoreBackend,
    pub file_root: PathBuf,
    pub sqlite_path: Option<PathBuf>,
    pub redis_url: Option<String>,
    pub s3_bucket: Option<String>,
}

impl Default for CertificateStoreConfig {
    fn default() -> Self {
        Self {
            backend: StoreBackend::File,
            file_root: PathBuf::from("certificates"),
            sqlite_path: None,
            redis_url: None,
            s3_bucket: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreBackend {
    File,
    SqliteReserved,
    RedisReserved,
    S3Reserved,
}
