use crate::cert_store::CertificateStore;
use crate::certificate::{CertificateRecord, CertificateValidator, StoredCertificate};
use crate::error::{CertificateError, TlsError};
use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateMaterial {
    pub certificate_pem: String,
    pub private_key_pem: String,
    pub record: CertificateRecord,
}

impl From<StoredCertificate> for CertificateMaterial {
    fn from(value: StoredCertificate) -> Self {
        Self {
            certificate_pem: value.certificate_pem,
            private_key_pem: value.private_key_pem,
            record: value.record,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpireCheck {
    pub domain: String,
    pub expires_at: DateTime<Utc>,
    pub days_remaining: i64,
    pub within_threshold: bool,
}

pub trait TlsProvider {
    fn load_certificate(&self, domain: &str) -> Result<String, TlsError>;
    fn load_key(&self, domain: &str) -> Result<String, TlsError>;
    fn reload(&mut self, domain: &str) -> Result<CertificateMaterial, TlsError>;
    fn validate(&self, domain: &str) -> Result<(), TlsError>;
    fn check_expire(&self, domain: &str, threshold: Duration) -> Result<ExpireCheck, TlsError>;
}

#[derive(Debug, Clone)]
pub struct StoreBackedTlsProvider<S> {
    store: S,
    validator: CertificateValidator,
}

impl<S> StoreBackedTlsProvider<S> {
    pub fn new(store: S, validator: CertificateValidator) -> Self {
        Self { store, validator }
    }

    pub fn store(&self) -> &S {
        &self.store
    }
}

impl<S> TlsProvider for StoreBackedTlsProvider<S>
where
    S: CertificateStore,
{
    fn load_certificate(&self, domain: &str) -> Result<String, TlsError> {
        Ok(self.load(domain)?.certificate_pem)
    }

    fn load_key(&self, domain: &str) -> Result<String, TlsError> {
        Ok(self.load(domain)?.private_key_pem)
    }

    fn reload(&mut self, domain: &str) -> Result<CertificateMaterial, TlsError> {
        let stored = self.load(domain)?;
        self.validator.validate_record(&stored.record, domain)?;
        Ok(stored.into())
    }

    fn validate(&self, domain: &str) -> Result<(), TlsError> {
        let stored = self.load(domain)?;
        self.validator.validate_record(&stored.record, domain)?;
        Ok(())
    }

    fn check_expire(&self, domain: &str, threshold: Duration) -> Result<ExpireCheck, TlsError> {
        let stored = self.load(domain)?;
        let now = Utc::now();
        let remaining = stored.record.expire_time.signed_duration_since(now);
        Ok(ExpireCheck {
            domain: domain.to_string(),
            expires_at: stored.record.expire_time,
            days_remaining: remaining.num_days(),
            within_threshold: remaining <= threshold,
        })
    }
}

impl<S> StoreBackedTlsProvider<S>
where
    S: CertificateStore,
{
    fn load(&self, domain: &str) -> Result<StoredCertificate, CertificateError> {
        self.store.load(domain)
    }
}
