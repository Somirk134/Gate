use crate::config::{DomainConfig, ValidationMode};
use crate::error::ValidateError;
use crate::model::{Alias, Host};
use crate::repository::DomainRepository;

const MAX_HOST_LENGTH: usize = 253;
const MAX_LABEL_LENGTH: usize = 63;
const WILDCARD_PREFIX: &str = "*.";

pub trait DomainValidator: Send + Sync {
    fn validate_host(&self, host: &str, config: &DomainConfig) -> Result<Host, ValidateError>;
    fn validate_alias(&self, alias: &str, config: &DomainConfig) -> Result<Alias, ValidateError>;
    fn validate_unique(
        &self,
        repository: &dyn DomainRepository,
        host: &Host,
    ) -> Result<(), ValidateError>;
}

#[derive(Clone, Default)]
pub struct RfcDomainValidator;

impl RfcDomainValidator {
    pub fn new() -> Self {
        Self
    }
}

impl DomainValidator for RfcDomainValidator {
    fn validate_host(&self, host: &str, config: &DomainConfig) -> Result<Host, ValidateError> {
        let normalized = Host::new(host)?;
        let value = normalized.as_str();

        validate_length(value)?;
        validate_wildcard(value, config)?;
        validate_international(value, config)?;

        let value_without_wildcard = match value.strip_prefix(WILDCARD_PREFIX) {
            Some(stripped) => stripped,
            None => value,
        };
        if config.is_reserved_domain(value_without_wildcard) {
            return Err(ValidateError::ReservedDomain(value.to_string()));
        }

        validate_labels(value_without_wildcard, &config.validation_mode)?;
        Ok(normalized)
    }

    fn validate_alias(&self, alias: &str, config: &DomainConfig) -> Result<Alias, ValidateError> {
        let host = self.validate_host(alias, config)?;
        Ok(Alias::new(host.as_str())?)
    }

    fn validate_unique(
        &self,
        repository: &dyn DomainRepository,
        host: &Host,
    ) -> Result<(), ValidateError> {
        match repository.exists(host) {
            Ok(false) => Ok(()),
            Ok(true) => Err(ValidateError::DuplicateHost(host.to_string())),
            Err(error) => Err(ValidateError::RepositoryUnavailable(error.to_string())),
        }
    }
}

fn validate_length(value: &str) -> Result<(), ValidateError> {
    let length = value.len();
    if length > MAX_HOST_LENGTH {
        return Err(ValidateError::HostTooLong {
            max: MAX_HOST_LENGTH,
            actual: length,
        });
    }
    Ok(())
}

fn validate_wildcard(value: &str, config: &DomainConfig) -> Result<(), ValidateError> {
    if !value.contains('*') {
        return Ok(());
    }

    if !config.allow_wildcard {
        return Err(ValidateError::WildcardDisabled);
    }

    if !value.starts_with(WILDCARD_PREFIX) || value[WILDCARD_PREFIX.len()..].contains('*') {
        return Err(ValidateError::InvalidWildcard(value.to_string()));
    }

    Ok(())
}

fn validate_international(value: &str, config: &DomainConfig) -> Result<(), ValidateError> {
    if value.is_ascii() {
        return Ok(());
    }

    if !config.allow_international {
        return Err(ValidateError::InternationalDomainDisabled);
    }

    Err(ValidateError::InternationalDomainReserved)
}

fn validate_labels(value: &str, mode: &ValidationMode) -> Result<(), ValidateError> {
    for label in value.split('.') {
        if label.is_empty() {
            return Err(ValidateError::EmptyLabel);
        }

        if label.len() > MAX_LABEL_LENGTH {
            return Err(ValidateError::LabelTooLong {
                max: MAX_LABEL_LENGTH,
                actual: label.len(),
            });
        }

        validate_label_chars(label, mode)?;
        validate_label_edges(label)?;
    }

    Ok(())
}

fn validate_label_chars(label: &str, mode: &ValidationMode) -> Result<(), ValidateError> {
    for character in label.chars() {
        let valid = character.is_ascii_alphanumeric()
            || character == '-'
            || matches!(mode, ValidationMode::Relaxed) && character == '_';

        if !valid {
            return Err(ValidateError::InvalidCharacter { character });
        }
    }
    Ok(())
}

fn validate_label_edges(label: &str) -> Result<(), ValidateError> {
    if label.starts_with('-') || label.ends_with('-') {
        return Err(ValidateError::InvalidLabel(label.to_string()));
    }
    Ok(())
}
