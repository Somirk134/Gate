use super::{CertificateAlgorithm, CertificateParser, CertificateRecord};
use crate::error::CertificateError;
use chrono::Utc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateValidator {
    allowed_algorithms: Vec<CertificateAlgorithm>,
}

impl Default for CertificateValidator {
    fn default() -> Self {
        Self {
            allowed_algorithms: vec![
                CertificateAlgorithm::Rsa,
                CertificateAlgorithm::EcdsaP256,
                CertificateAlgorithm::EcdsaP384,
                CertificateAlgorithm::Ed25519,
            ],
        }
    }
}

impl CertificateValidator {
    pub fn new(allowed_algorithms: Vec<CertificateAlgorithm>) -> Self {
        Self { allowed_algorithms }
    }

    pub fn validate_pem_for_domain(
        &self,
        pem_text: &str,
        domain: &str,
    ) -> Result<CertificateRecord, CertificateError> {
        let record = CertificateParser::parse_pem(domain, pem_text)?;
        self.validate_record(&record, domain)?;
        Ok(record)
    }

    pub fn validate_record(
        &self,
        record: &CertificateRecord,
        domain: &str,
    ) -> Result<(), CertificateError> {
        self.validate_not_expired(record)?;
        self.validate_domain(record, domain)?;
        self.validate_algorithm(record)?;
        Ok(())
    }

    pub fn validate_not_expired(&self, record: &CertificateRecord) -> Result<(), CertificateError> {
        if record.expire_time <= Utc::now() {
            return Err(CertificateError::Expired {
                domain: record.domain.clone(),
                expired_at: record.expire_time,
            });
        }

        Ok(())
    }

    pub fn validate_domain(
        &self,
        record: &CertificateRecord,
        domain: &str,
    ) -> Result<(), CertificateError> {
        if domain_matches(&record.domain, domain)
            || record.san.iter().any(|name| domain_matches(name, domain))
        {
            return Ok(());
        }

        Err(CertificateError::DomainMismatch {
            domain: domain.to_string(),
        })
    }

    pub fn validate_algorithm(&self, record: &CertificateRecord) -> Result<(), CertificateError> {
        if self.allowed_algorithms.contains(&record.algorithm)
            && record.algorithm.is_supported_for_tls()
        {
            return Ok(());
        }

        Err(CertificateError::InvalidAlgorithm {
            algorithm: record.algorithm.name().to_string(),
        })
    }
}

fn domain_matches(pattern: &str, domain: &str) -> bool {
    let pattern = pattern.trim_end_matches('.').to_ascii_lowercase();
    let domain = domain.trim_end_matches('.').to_ascii_lowercase();

    if pattern == domain {
        return true;
    }

    let Some(suffix) = pattern.strip_prefix("*.") else {
        return false;
    };

    if !domain.ends_with(suffix) {
        return false;
    }

    let prefix = domain.trim_end_matches(suffix).trim_end_matches('.');
    !prefix.is_empty() && !prefix.contains('.')
}
