use super::{CertificateAlgorithm, CertificateFingerprint, CertificateRecord, CertificateStatus};
use crate::crypto::sha256_fingerprint;
use crate::error::CertificateError;
use chrono::{TimeZone, Utc};
use x509_parser::extensions::GeneralName;
use x509_parser::prelude::*;

pub struct CertificateParser;

impl CertificateParser {
    pub fn parse_pem(
        domain: impl Into<String>,
        pem_text: &str,
    ) -> Result<CertificateRecord, CertificateError> {
        let domain = domain.into();
        let pem = ::pem::parse(pem_text).map_err(|source| CertificateError::InvalidPem {
            reason: source.to_string(),
        })?;

        let (_, certificate) =
            parse_x509_certificate(pem.contents()).map_err(|source| CertificateError::Parse {
                reason: source.to_string(),
            })?;

        let create_time = Utc
            .timestamp_opt(certificate.validity().not_before.timestamp(), 0)
            .single()
            .ok_or_else(|| CertificateError::Parse {
                reason: "invalid not_before timestamp".to_string(),
            })?;
        let expire_time = Utc
            .timestamp_opt(certificate.validity().not_after.timestamp(), 0)
            .single()
            .ok_or_else(|| CertificateError::Parse {
                reason: "invalid not_after timestamp".to_string(),
            })?;

        let san = subject_alt_names(&certificate)?;
        let status = if expire_time <= Utc::now() {
            CertificateStatus::Expired
        } else {
            CertificateStatus::Active
        };

        Ok(CertificateRecord {
            domain,
            issuer: certificate.issuer().to_string(),
            expire_time,
            create_time,
            renew_time: None,
            status,
            fingerprint: CertificateFingerprint {
                sha256: sha256_fingerprint(pem.contents()),
            },
            algorithm: public_key_algorithm(&certificate),
            san,
            cert_path: None,
            key_path: None,
            serial_number: Some(certificate.raw_serial_as_string()),
            last_error: None,
        })
    }
}

fn subject_alt_names(certificate: &X509Certificate<'_>) -> Result<Vec<String>, CertificateError> {
    let Some(extension) =
        certificate
            .subject_alternative_name()
            .map_err(|source| CertificateError::Parse {
                reason: source.to_string(),
            })?
    else {
        return Ok(Vec::new());
    };

    Ok(extension
        .value
        .general_names
        .iter()
        .filter_map(|name| match name {
            GeneralName::DNSName(value) => Some((*value).to_string()),
            _ => None,
        })
        .collect())
}

fn public_key_algorithm(certificate: &X509Certificate<'_>) -> CertificateAlgorithm {
    let algorithm = certificate.public_key().algorithm.algorithm.to_id_string();

    match algorithm.as_str() {
        "1.2.840.113549.1.1.1" => CertificateAlgorithm::Rsa,
        "1.2.840.10045.2.1" => {
            let parameters = format!("{:?}", certificate.public_key().algorithm.parameters);
            if parameters.contains("1.3.132.0.34") {
                CertificateAlgorithm::EcdsaP384
            } else {
                CertificateAlgorithm::EcdsaP256
            }
        }
        "1.3.101.112" => CertificateAlgorithm::Ed25519,
        value => CertificateAlgorithm::Unknown(value.to_string()),
    }
}
