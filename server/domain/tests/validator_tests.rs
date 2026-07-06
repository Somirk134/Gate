use crate::config::DomainConfig;
use crate::error::ValidateError;
use crate::model::{Domain, DomainId, Host};
use crate::repository::{DomainRepository, MemoryRepository};
use crate::validator::{DomainValidator, RfcDomainValidator};

#[test]
fn validator_accepts_rfc_host() -> Result<(), Box<dyn std::error::Error>> {
    let validator = RfcDomainValidator::new();
    let host = validator.validate_host("api.gate.dev", &DomainConfig::default())?;
    assert_eq!(host.as_str(), "api.gate.dev");
    Ok(())
}

#[test]
fn validator_rejects_invalid_character() {
    let validator = RfcDomainValidator::new();
    let result = validator.validate_host("bad_host.gate.dev", &DomainConfig::default());
    assert!(matches!(result, Err(ValidateError::InvalidCharacter { .. })));
}

#[test]
fn validator_rejects_reserved_domain() {
    let validator = RfcDomainValidator::new();
    let result = validator.validate_host("api.localhost", &DomainConfig::default());
    assert!(matches!(result, Err(ValidateError::ReservedDomain(_))));
}

#[test]
fn validator_reserves_wildcard_policy() -> Result<(), Box<dyn std::error::Error>> {
    let validator = RfcDomainValidator::new();
    let disabled = validator.validate_host("*.gate.dev", &DomainConfig::default());
    assert!(matches!(disabled, Err(ValidateError::WildcardDisabled)));

    let config = DomainConfig::builder().allow_wildcard(true).build();
    let wildcard = validator.validate_host("*.gate.dev", &config)?;
    assert!(wildcard.is_wildcard());
    Ok(())
}

#[test]
fn validator_detects_duplicates_through_repository() -> Result<(), Box<dyn std::error::Error>> {
    let repository = MemoryRepository::new();
    let validator = RfcDomainValidator::new();
    let host = Host::new("api.gate.dev")?;
    let domain = Domain::builder(DomainId::new("domain-1")?, host.clone()).build()?;
    repository.create(domain)?;

    let result = validator.validate_unique(&repository, &host);
    assert!(matches!(result, Err(ValidateError::DuplicateHost(_))));
    Ok(())
}
