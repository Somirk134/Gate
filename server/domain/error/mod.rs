use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Common behavior for all domain infrastructure errors.
pub trait DomainFailure: Error + Send + Sync {
    fn code(&self) -> &'static str;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomainError {
    NotFound(String),
    AlreadyExists(String),
    LimitExceeded { max: usize },
    Validation(ValidateError),
    Bind(BindError),
    Resolve(ResolveError),
    Dns(DnsError),
    Storage(StorageError),
    InvalidOperation(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidateError {
    EmptyHost,
    EmptyAlias,
    InvalidId(String),
    HostTooLong { max: usize, actual: usize },
    LabelTooLong { max: usize, actual: usize },
    EmptyLabel,
    InvalidCharacter { character: char },
    InvalidLabel(String),
    ReservedDomain(String),
    WildcardDisabled,
    InvalidWildcard(String),
    InternationalDomainDisabled,
    InternationalDomainReserved,
    DuplicateHost(String),
    RepositoryUnavailable(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindError {
    AlreadyBound {
        domain_id: String,
        tunnel_id: String,
    },
    NotBound(String),
    DisabledDomain(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolveError {
    InvalidHost(String),
    HostNotFound(String),
    DomainDisabled(String),
    UnboundHost(String),
    Storage(StorageError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DnsError {
    RecordNotFound(String),
    UnsupportedRecord(String),
    ResolverUnavailable(String),
    InvalidResponse(String),
    Storage(StorageError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StorageError {
    LockPoisoned(&'static str),
    DuplicateKey(String),
    NotFound(String),
    Unavailable(String),
}

impl Display for DomainError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(id) => write!(formatter, "domain not found: {id}"),
            Self::AlreadyExists(host) => write!(formatter, "domain already exists: {host}"),
            Self::LimitExceeded { max } => write!(formatter, "domain limit exceeded: max {max}"),
            Self::Validation(error) => Display::fmt(error, formatter),
            Self::Bind(error) => Display::fmt(error, formatter),
            Self::Resolve(error) => Display::fmt(error, formatter),
            Self::Dns(error) => Display::fmt(error, formatter),
            Self::Storage(error) => Display::fmt(error, formatter),
            Self::InvalidOperation(message) => write!(formatter, "invalid domain operation: {message}"),
        }
    }
}

impl Display for ValidateError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyHost => write!(formatter, "host is empty"),
            Self::EmptyAlias => write!(formatter, "alias is empty"),
            Self::InvalidId(id) => write!(formatter, "invalid id: {id}"),
            Self::HostTooLong { max, actual } => {
                write!(formatter, "host is too long: max {max}, actual {actual}")
            }
            Self::LabelTooLong { max, actual } => {
                write!(formatter, "domain label is too long: max {max}, actual {actual}")
            }
            Self::EmptyLabel => write!(formatter, "domain label is empty"),
            Self::InvalidCharacter { character } => {
                write!(formatter, "host contains invalid character: {character}")
            }
            Self::InvalidLabel(label) => write!(formatter, "invalid domain label: {label}"),
            Self::ReservedDomain(host) => write!(formatter, "reserved domain is not allowed: {host}"),
            Self::WildcardDisabled => write!(formatter, "wildcard domain is disabled"),
            Self::InvalidWildcard(host) => write!(formatter, "invalid wildcard domain: {host}"),
            Self::InternationalDomainDisabled => {
                write!(formatter, "international domain validation is disabled")
            }
            Self::InternationalDomainReserved => {
                write!(formatter, "international domain validation is reserved")
            }
            Self::DuplicateHost(host) => write!(formatter, "duplicate host: {host}"),
            Self::RepositoryUnavailable(message) => {
                write!(formatter, "repository unavailable during validation: {message}")
            }
        }
    }
}

impl Display for BindError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyBound {
                domain_id,
                tunnel_id,
            } => write!(
                formatter,
                "domain {domain_id} is already bound to tunnel {tunnel_id}"
            ),
            Self::NotBound(domain_id) => write!(formatter, "domain is not bound: {domain_id}"),
            Self::DisabledDomain(domain_id) => {
                write!(formatter, "disabled domain cannot be bound: {domain_id}")
            }
        }
    }
}

impl Display for ResolveError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHost(host) => write!(formatter, "invalid host: {host}"),
            Self::HostNotFound(host) => write!(formatter, "host not found: {host}"),
            Self::DomainDisabled(id) => write!(formatter, "domain is disabled: {id}"),
            Self::UnboundHost(host) => write!(formatter, "host is not bound to a tunnel: {host}"),
            Self::Storage(error) => Display::fmt(error, formatter),
        }
    }
}

impl Display for DnsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::RecordNotFound(host) => write!(formatter, "DNS record not found: {host}"),
            Self::UnsupportedRecord(record_type) => {
                write!(formatter, "unsupported DNS record type: {record_type}")
            }
            Self::ResolverUnavailable(message) => write!(formatter, "DNS resolver unavailable: {message}"),
            Self::InvalidResponse(message) => write!(formatter, "invalid DNS response: {message}"),
            Self::Storage(error) => Display::fmt(error, formatter),
        }
    }
}

impl Display for StorageError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LockPoisoned(name) => write!(formatter, "storage lock poisoned: {name}"),
            Self::DuplicateKey(key) => write!(formatter, "duplicate storage key: {key}"),
            Self::NotFound(key) => write!(formatter, "storage key not found: {key}"),
            Self::Unavailable(message) => write!(formatter, "storage unavailable: {message}"),
        }
    }
}

impl Error for DomainError {}
impl Error for ValidateError {}
impl Error for BindError {}
impl Error for ResolveError {}
impl Error for DnsError {}
impl Error for StorageError {}

impl DomainFailure for DomainError {
    fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "DOMAIN_NOT_FOUND",
            Self::AlreadyExists(_) => "DOMAIN_ALREADY_EXISTS",
            Self::LimitExceeded { .. } => "DOMAIN_LIMIT_EXCEEDED",
            Self::Validation(_) => "DOMAIN_VALIDATION_ERROR",
            Self::Bind(_) => "DOMAIN_BIND_ERROR",
            Self::Resolve(_) => "DOMAIN_RESOLVE_ERROR",
            Self::Dns(_) => "DOMAIN_DNS_ERROR",
            Self::Storage(_) => "DOMAIN_STORAGE_ERROR",
            Self::InvalidOperation(_) => "DOMAIN_INVALID_OPERATION",
        }
    }
}

impl DomainFailure for ValidateError {
    fn code(&self) -> &'static str {
        "DOMAIN_VALIDATE_ERROR"
    }
}

impl DomainFailure for BindError {
    fn code(&self) -> &'static str {
        "DOMAIN_BIND_ERROR"
    }
}

impl DomainFailure for ResolveError {
    fn code(&self) -> &'static str {
        "DOMAIN_RESOLVE_ERROR"
    }
}

impl DomainFailure for DnsError {
    fn code(&self) -> &'static str {
        "DOMAIN_DNS_ERROR"
    }
}

impl DomainFailure for StorageError {
    fn code(&self) -> &'static str {
        "DOMAIN_STORAGE_ERROR"
    }
}

impl From<ValidateError> for DomainError {
    fn from(error: ValidateError) -> Self {
        Self::Validation(error)
    }
}

impl From<BindError> for DomainError {
    fn from(error: BindError) -> Self {
        Self::Bind(error)
    }
}

impl From<ResolveError> for DomainError {
    fn from(error: ResolveError) -> Self {
        Self::Resolve(error)
    }
}

impl From<DnsError> for DomainError {
    fn from(error: DnsError) -> Self {
        Self::Dns(error)
    }
}

impl From<StorageError> for DomainError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

impl From<StorageError> for ResolveError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}

impl From<StorageError> for DnsError {
    fn from(error: StorageError) -> Self {
        Self::Storage(error)
    }
}
