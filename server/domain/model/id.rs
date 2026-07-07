use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::error::ValidateError;

#[derive(Clone, Debug, Eq, Ord, PartialOrd)]
pub struct DomainId(String);

#[derive(Clone, Debug, Eq, Ord, PartialOrd)]
pub struct TunnelId(String);

impl DomainId {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidateError> {
        let value = value.into();
        validate_id("domain", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TunnelId {
    pub fn new(value: impl Into<String>) -> Result<Self, ValidateError> {
        let value = value.into();
        validate_id("tunnel", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn validate_id(kind: &str, value: &str) -> Result<(), ValidateError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed != value {
        return Err(ValidateError::InvalidId(format!("{kind}:{value}")));
    }

    let valid = value.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | ':' | '.')
    });

    if valid {
        Ok(())
    } else {
        Err(ValidateError::InvalidId(format!("{kind}:{value}")))
    }
}

impl PartialEq for DomainId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Hash for DomainId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialEq for TunnelId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Hash for TunnelId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Display for DomainId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Display for TunnelId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
