use std::collections::BTreeSet;

/// Default maximum number of managed domains.
pub const DEFAULT_MAX_DOMAINS: usize = 1_000;

/// Default DNS TTL used by mock and future DNS checkers.
pub const DEFAULT_TTL_SECONDS: u32 = 300;

/// Domain syntax validation strictness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationMode {
    /// RFC-oriented host validation.
    Rfc,
    /// Reserved for operators that need a softer migration mode.
    Relaxed,
    /// Strict production validation.
    Strict,
}

/// Storage backend kind.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StorageKind {
    Memory,
    Sqlite,
    RedisReserved,
    JsonReserved,
    FileReserved,
}

/// Domain infrastructure configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomainConfig {
    pub max_domains: usize,
    pub allow_wildcard: bool,
    pub allow_international: bool,
    pub enable_dns_check: bool,
    pub default_ttl: u32,
    pub validation_mode: ValidationMode,
    pub storage_kind: StorageKind,
    pub reserved_domains: BTreeSet<String>,
    pub server_addresses: Vec<String>,
}

impl DomainConfig {
    /// Creates a builder with production-safe defaults.
    pub fn builder() -> DomainConfigBuilder {
        DomainConfigBuilder::default()
    }

    /// Returns true when the host exactly matches or belongs to a reserved suffix.
    pub fn is_reserved_domain(&self, host: &str) -> bool {
        let normalized = host.trim_end_matches('.').to_ascii_lowercase();
        self.reserved_domains.iter().any(|reserved| {
            normalized == *reserved || normalized.ends_with(&format!(".{reserved}"))
        })
    }
}

impl Default for DomainConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Builder for [`DomainConfig`].
#[derive(Clone, Debug)]
pub struct DomainConfigBuilder {
    max_domains: usize,
    allow_wildcard: bool,
    allow_international: bool,
    enable_dns_check: bool,
    default_ttl: u32,
    validation_mode: ValidationMode,
    storage_kind: StorageKind,
    reserved_domains: BTreeSet<String>,
    server_addresses: Vec<String>,
}

impl DomainConfigBuilder {
    pub fn max_domains(mut self, max_domains: usize) -> Self {
        self.max_domains = max_domains;
        self
    }

    pub fn allow_wildcard(mut self, allow_wildcard: bool) -> Self {
        self.allow_wildcard = allow_wildcard;
        self
    }

    pub fn allow_international(mut self, allow_international: bool) -> Self {
        self.allow_international = allow_international;
        self
    }

    pub fn enable_dns_check(mut self, enable_dns_check: bool) -> Self {
        self.enable_dns_check = enable_dns_check;
        self
    }

    pub fn default_ttl(mut self, default_ttl: u32) -> Self {
        self.default_ttl = default_ttl;
        self
    }

    pub fn validation_mode(mut self, validation_mode: ValidationMode) -> Self {
        self.validation_mode = validation_mode;
        self
    }

    pub fn storage_kind(mut self, storage_kind: StorageKind) -> Self {
        self.storage_kind = storage_kind;
        self
    }

    pub fn reserved_domain(mut self, reserved_domain: impl Into<String>) -> Self {
        self.reserved_domains
            .insert(reserved_domain.into().to_ascii_lowercase());
        self
    }

    pub fn server_address(mut self, server_address: impl Into<String>) -> Self {
        self.server_addresses.push(server_address.into());
        self
    }

    pub fn build(self) -> DomainConfig {
        DomainConfig {
            max_domains: self.max_domains,
            allow_wildcard: self.allow_wildcard,
            allow_international: self.allow_international,
            enable_dns_check: self.enable_dns_check,
            default_ttl: self.default_ttl,
            validation_mode: self.validation_mode,
            storage_kind: self.storage_kind,
            reserved_domains: self.reserved_domains,
            server_addresses: self.server_addresses,
        }
    }
}

impl Default for DomainConfigBuilder {
    fn default() -> Self {
        let mut reserved_domains = BTreeSet::new();
        for reserved in [
            "localhost",
            "local",
            "test",
            "example",
            "invalid",
            "example.com",
            "example.net",
            "example.org",
        ] {
            reserved_domains.insert(reserved.to_string());
        }

        Self {
            max_domains: DEFAULT_MAX_DOMAINS,
            allow_wildcard: false,
            allow_international: false,
            enable_dns_check: false,
            default_ttl: DEFAULT_TTL_SECONDS,
            validation_mode: ValidationMode::Rfc,
            storage_kind: StorageKind::Memory,
            reserved_domains,
            server_addresses: Vec::new(),
        }
    }
}
