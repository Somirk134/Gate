use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::error::ValidateError;

#[derive(Clone, Debug, Eq, Ord, PartialOrd)]
pub struct Host(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alias(Host);

impl Host {
    /// Normalizes a host without performing full RFC validation.
    ///
    /// Full validation belongs to the validator layer so future wildcard and IDN
    /// policies can be swapped without changing the model.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ValidateError> {
        let normalized = normalize_host(value.as_ref());
        if normalized.is_empty() {
            return Err(ValidateError::EmptyHost);
        }
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn labels(&self) -> impl Iterator<Item = &str> {
        self.0.split('.')
    }

    pub fn is_wildcard(&self) -> bool {
        self.0.starts_with("*.")
    }
}

impl Alias {
    pub fn new(value: impl AsRef<str>) -> Result<Self, ValidateError> {
        let host = Host::new(value)?;
        if host.as_str().is_empty() {
            return Err(ValidateError::EmptyAlias);
        }
        Ok(Self(host))
    }

    pub fn host(&self) -> &Host {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

fn normalize_host(value: &str) -> String {
    value.trim().trim_end_matches('.').to_ascii_lowercase()
}

impl PartialEq for Host {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Hash for Host {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Hash for Alias {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Display for Host {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Display for Alias {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
