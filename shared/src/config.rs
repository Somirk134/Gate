//! Unified configuration center primitives.
//!
//! This module defines a shared configuration layer and the infrastructure
//! needed to load, validate, watch, export, import, and migrate configuration
//! without coupling the shared kernel back to runtime or business crates.

use crate::error::ConfigError;
use crate::health::HealthConfig;
use crate::logging::LoggingConfig;
use config as config_crate;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};

pub const CURRENT_CONFIG_VERSION: u32 = 1;
pub const DEFAULT_ENV_PREFIX: &str = "GATE";
pub const DEFAULT_ENV_SEPARATOR: &str = "__";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigSource {
    Default,
    File(PathBuf),
    Environment { prefix: String },
    Memory { name: String },
    Cli,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConfigPriority {
    Default,
    File,
    Memory,
    Environment,
    Cli,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CliConfig {
    pub config_path: Option<PathBuf>,
    pub environment_prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub version: u32,
    pub global: GlobalConfig,
    pub runtime: RuntimeConfig,
    pub http: HttpConfig,
    pub https: HttpsConfig,
    pub tls: TlsConfig,
    pub domain: DomainConfig,
    pub dashboard: DashboardConfig,
    pub client: ClientConfig,
    pub logging: LoggingConfig,
    pub health: HealthConfig,
}

impl AppConfig {
    pub fn validate_startup(&self) -> Result<(), ConfigError> {
        self.validate()
    }

    pub fn export_json(&self) -> Result<String, ConfigError> {
        serde_json::to_string_pretty(self).map_err(|source| ConfigError::Parse {
            source_name: "app_config".to_string(),
            message: source.to_string(),
        })
    }

    pub fn import_json(input: &str) -> Result<Self, ConfigError> {
        let overlay =
            serde_json::from_str::<Value>(input).map_err(|source| ConfigError::Parse {
                source_name: "json".to_string(),
                message: source.to_string(),
            })?;
        UnifiedConfigCenter::new().load_value(overlay)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CURRENT_CONFIG_VERSION,
            global: GlobalConfig::default(),
            runtime: RuntimeConfig::default(),
            http: HttpConfig::default(),
            https: HttpsConfig::default(),
            tls: TlsConfig::default(),
            domain: DomainConfig::default(),
            dashboard: DashboardConfig::default(),
            client: ClientConfig::default(),
            logging: LoggingConfig::default(),
            health: HealthConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GlobalConfig {
    pub app_name: String,
    pub environment: RuntimeEnvironment,
    pub instance_id: Option<String>,
    pub data_dir: PathBuf,
    pub log_dir: PathBuf,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            app_name: "gate".to_string(),
            environment: RuntimeEnvironment::Development,
            instance_id: None,
            data_dir: PathBuf::from("data"),
            log_dir: PathBuf::from("logs"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeEnvironment {
    Development,
    Test,
    Staging,
    Production,
}

impl Default for RuntimeEnvironment {
    fn default() -> Self {
        Self::Development
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RuntimeConfig {
    pub instance_name: String,
    pub worker_threads: usize,
    pub max_tasks: usize,
    pub max_sessions: usize,
    pub graceful_shutdown_timeout_seconds: u64,
    pub monitor_interval_seconds: u64,
    pub cleanup_interval_seconds: u64,
    pub tcp_nodelay: bool,
    pub max_connections: usize,
    pub buffer_size_bytes: usize,
    pub dynamic_buffer_limit_bytes: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            instance_name: "gate-server".to_string(),
            worker_threads: 4,
            max_tasks: 4096,
            max_sessions: 4096,
            graceful_shutdown_timeout_seconds: 30,
            monitor_interval_seconds: 1,
            cleanup_interval_seconds: 30,
            tcp_nodelay: true,
            max_connections: 4096,
            buffer_size_bytes: 64 * 1024,
            dynamic_buffer_limit_bytes: 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpConfig {
    pub enabled: bool,
    pub listen_addr: String,
    pub max_header_bytes: usize,
    pub recent_log_limit: usize,
    pub route_limit: usize,
    pub forwarded_proto: String,
    pub upgrade_reserved: bool,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            listen_addr: "127.0.0.1:8080".to_string(),
            max_header_bytes: 64 * 1024,
            recent_log_limit: 512,
            route_limit: 1024,
            forwarded_proto: "http".to_string(),
            upgrade_reserved: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpsConfig {
    pub enabled: bool,
    pub listen_addr: String,
    pub http_redirect: bool,
    pub minimum_tls_version: TlsProtocolVersion,
    pub preferred_tls_version: Option<TlsProtocolVersion>,
    pub hsts_enabled: bool,
    pub ocsp_enabled: bool,
    pub certificate_domains: Vec<String>,
}

impl Default for HttpsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            listen_addr: "127.0.0.1:8443".to_string(),
            http_redirect: false,
            minimum_tls_version: TlsProtocolVersion::Tls12,
            preferred_tls_version: Some(TlsProtocolVersion::Tls13),
            hsts_enabled: false,
            ocsp_enabled: false,
            certificate_domains: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TlsProtocolVersion {
    Tls12,
    Tls13,
}

impl Default for TlsProtocolVersion {
    fn default() -> Self {
        Self::Tls13
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub auto_renew: bool,
    pub auto_apply: bool,
    pub renew_before_days: i64,
    pub preferred_chain: Option<String>,
    pub challenge_type: ChallengeType,
    pub dns_provider: Option<String>,
    pub store_path: PathBuf,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: None,
            key_path: None,
            auto_renew: true,
            auto_apply: false,
            renew_before_days: 30,
            preferred_chain: None,
            challenge_type: ChallengeType::Http01,
            dns_provider: None,
            store_path: PathBuf::from("certificates"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeType {
    Http01,
    Dns01,
    TlsAlpn01,
}

impl Default for ChallengeType {
    fn default() -> Self {
        Self::Http01
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DomainConfig {
    pub max_domains: usize,
    pub allow_wildcard: bool,
    pub allow_international: bool,
    pub enable_dns_check: bool,
    pub default_ttl_seconds: u32,
    pub validation_mode: DomainValidationMode,
    pub storage_kind: DomainStorageKind,
    pub reserved_domains: BTreeSet<String>,
    pub server_addresses: Vec<String>,
}

impl Default for DomainConfig {
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
            max_domains: 1_000,
            allow_wildcard: false,
            allow_international: false,
            enable_dns_check: false,
            default_ttl_seconds: 300,
            validation_mode: DomainValidationMode::Rfc,
            storage_kind: DomainStorageKind::Sqlite,
            reserved_domains,
            server_addresses: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainValidationMode {
    Rfc,
    Relaxed,
    Strict,
}

impl Default for DomainValidationMode {
    fn default() -> Self {
        Self::Rfc
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainStorageKind {
    Sqlite,
}

impl Default for DomainStorageKind {
    fn default() -> Self {
        Self::Sqlite
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DashboardConfig {
    pub enabled: bool,
    pub refresh_interval_seconds: u64,
    pub realtime_points: usize,
    pub trend_points: usize,
    pub recent_log_limit: usize,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            refresh_interval_seconds: 5,
            realtime_points: 60,
            trend_points: 24,
            recent_log_limit: 512,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClientConfig {
    pub server_addr: String,
    pub auth_token: String,
    pub theme: String,
    pub language: String,
    pub auto_connect: bool,
    pub minimize_to_tray: bool,
    pub config_namespace: String,
    pub shortcut_profile: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_addr: String::new(),
            auth_token: String::new(),
            theme: "light".to_string(),
            language: "en".to_string(),
            auto_connect: false,
            minimize_to_tray: true,
            config_namespace: "configuration".to_string(),
            shortcut_profile: "default".to_string(),
        }
    }
}

pub trait ValidateConfig {
    fn validate(&self) -> Result<(), ConfigError>;
}

impl ValidateConfig for AppConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        validate_positive("version", self.version)?;
        self.global.validate()?;
        self.runtime.validate()?;
        self.http.validate()?;
        self.https.validate()?;
        self.tls.validate()?;
        self.domain.validate()?;
        self.dashboard.validate()?;
        self.client.validate()?;
        Ok(())
    }
}

impl ValidateConfig for GlobalConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        validate_non_empty("global.app_name", &self.app_name)
    }
}

impl ValidateConfig for RuntimeConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        validate_non_empty("runtime.instance_name", &self.instance_name)?;
        validate_positive("runtime.worker_threads", self.worker_threads)?;
        validate_positive("runtime.max_tasks", self.max_tasks)?;
        validate_positive("runtime.max_sessions", self.max_sessions)?;
        validate_positive(
            "runtime.graceful_shutdown_timeout_seconds",
            self.graceful_shutdown_timeout_seconds,
        )?;
        validate_positive("runtime.max_connections", self.max_connections)?;
        validate_positive("runtime.buffer_size_bytes", self.buffer_size_bytes)?;
        if self.dynamic_buffer_limit_bytes < self.buffer_size_bytes {
            return validation_error(
                "runtime.dynamic_buffer_limit_bytes",
                "must be greater than or equal to runtime.buffer_size_bytes",
            );
        }
        Ok(())
    }
}

impl ValidateConfig for HttpConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        validate_socket_addr("http.listen_addr", &self.listen_addr)?;
        validate_positive("http.max_header_bytes", self.max_header_bytes)?;
        validate_positive("http.recent_log_limit", self.recent_log_limit)?;
        validate_positive("http.route_limit", self.route_limit)?;
        validate_non_empty("http.forwarded_proto", &self.forwarded_proto)?;
        Ok(())
    }
}

impl ValidateConfig for HttpsConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        validate_socket_addr("https.listen_addr", &self.listen_addr)?;
        if self.minimum_tls_version == TlsProtocolVersion::Tls13
            && self.preferred_tls_version == Some(TlsProtocolVersion::Tls12)
        {
            return validation_error(
                "https.preferred_tls_version",
                "cannot prefer TLS 1.2 when minimum TLS version is TLS 1.3",
            );
        }
        Ok(())
    }
}

impl ValidateConfig for TlsConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.enabled && self.cert_path.is_some() != self.key_path.is_some() {
            return validation_error(
                "tls.cert_path",
                "cert_path and key_path must be configured together",
            );
        }
        if self.challenge_type == ChallengeType::Dns01 && self.dns_provider.is_none() {
            return validation_error("tls.dns_provider", "dns_provider is required for DNS-01");
        }
        if self.renew_before_days < 0 {
            return validation_error("tls.renew_before_days", "must be non-negative");
        }
        Ok(())
    }
}

impl ValidateConfig for DomainConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        validate_positive("domain.max_domains", self.max_domains)?;
        validate_positive("domain.default_ttl_seconds", self.default_ttl_seconds)?;
        Ok(())
    }
}

impl ValidateConfig for DashboardConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        validate_positive(
            "dashboard.refresh_interval_seconds",
            self.refresh_interval_seconds,
        )?;
        validate_positive("dashboard.realtime_points", self.realtime_points)?;
        validate_positive("dashboard.trend_points", self.trend_points)?;
        validate_positive("dashboard.recent_log_limit", self.recent_log_limit)?;
        Ok(())
    }
}

impl ValidateConfig for ClientConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        validate_non_empty("client.theme", &self.theme)?;
        validate_non_empty("client.language", &self.language)?;
        validate_non_empty("client.config_namespace", &self.config_namespace)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigDocument {
    pub source: ConfigSource,
    pub priority: ConfigPriority,
    pub value: Value,
}

pub trait ConfigProvider: Send + Sync {
    fn name(&self) -> &str;
    fn source(&self) -> ConfigSource;
    fn priority(&self) -> ConfigPriority;
    fn load(&self) -> Result<ConfigDocument, ConfigError>;
}

#[derive(Debug, Clone)]
pub struct FileConfigProvider {
    path: PathBuf,
    required: bool,
    priority: ConfigPriority,
}

impl FileConfigProvider {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            required: true,
            priority: ConfigPriority::File,
        }
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

impl ConfigProvider for FileConfigProvider {
    fn name(&self) -> &str {
        "file"
    }

    fn source(&self) -> ConfigSource {
        ConfigSource::File(self.path.clone())
    }

    fn priority(&self) -> ConfigPriority {
        self.priority
    }

    fn load(&self) -> Result<ConfigDocument, ConfigError> {
        if !self.path.exists() {
            if self.required {
                return Err(ConfigError::SourceUnavailable {
                    source_name: self.path.display().to_string(),
                });
            }
            return Ok(ConfigDocument {
                source: self.source(),
                priority: self.priority,
                value: Value::Object(Map::new()),
            });
        }

        let settings = config_crate::Config::builder()
            .add_source(config_crate::File::from(self.path.as_path()))
            .build()
            .map_err(|source| ConfigError::Parse {
                source_name: self.path.display().to_string(),
                message: source.to_string(),
            })?;
        let value = settings
            .try_deserialize::<Value>()
            .map_err(|source| ConfigError::Parse {
                source_name: self.path.display().to_string(),
                message: source.to_string(),
            })?;
        Ok(ConfigDocument {
            source: self.source(),
            priority: self.priority,
            value,
        })
    }
}

#[derive(Debug, Clone)]
pub struct EnvironmentConfigProvider {
    prefix: String,
    separator: String,
    priority: ConfigPriority,
}

impl EnvironmentConfigProvider {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            separator: DEFAULT_ENV_SEPARATOR.to_string(),
            priority: ConfigPriority::Environment,
        }
    }

    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }
}

impl Default for EnvironmentConfigProvider {
    fn default() -> Self {
        Self::new(DEFAULT_ENV_PREFIX)
    }
}

impl ConfigProvider for EnvironmentConfigProvider {
    fn name(&self) -> &str {
        "environment"
    }

    fn source(&self) -> ConfigSource {
        ConfigSource::Environment {
            prefix: self.prefix.clone(),
        }
    }

    fn priority(&self) -> ConfigPriority {
        self.priority
    }

    fn load(&self) -> Result<ConfigDocument, ConfigError> {
        let mut root = Value::Object(Map::new());
        let prefix = format!("{}_", self.prefix);
        for (key, raw_value) in std::env::vars() {
            if !key.starts_with(&prefix) {
                continue;
            }
            let suffix = &key[prefix.len()..];
            let path = suffix
                .split(&self.separator)
                .filter(|segment| !segment.is_empty())
                .map(|segment| segment.to_ascii_lowercase())
                .collect::<Vec<_>>();
            if path.is_empty() {
                continue;
            }
            set_path(&mut root, &path, parse_env_value(&raw_value));
        }

        Ok(ConfigDocument {
            source: self.source(),
            priority: self.priority,
            value: root,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MemoryConfigProvider {
    name: String,
    value: Value,
    priority: ConfigPriority,
}

impl MemoryConfigProvider {
    pub fn new(name: impl Into<String>, value: Value) -> Self {
        Self {
            name: name.into(),
            value,
            priority: ConfigPriority::Memory,
        }
    }

    pub fn with_priority(mut self, priority: ConfigPriority) -> Self {
        self.priority = priority;
        self
    }
}

impl ConfigProvider for MemoryConfigProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn source(&self) -> ConfigSource {
        ConfigSource::Memory {
            name: self.name.clone(),
        }
    }

    fn priority(&self) -> ConfigPriority {
        self.priority
    }

    fn load(&self) -> Result<ConfigDocument, ConfigError> {
        Ok(ConfigDocument {
            source: self.source(),
            priority: self.priority,
            value: self.value.clone(),
        })
    }
}

pub trait ConfigCenter: Send + Sync {
    fn load(&self, sources: &[ConfigSource]) -> Result<AppConfig, ConfigError>;

    fn priorities(&self) -> &[ConfigPriority];
}

pub struct UnifiedConfigCenter {
    providers: Vec<Box<dyn ConfigProvider>>,
    priorities: Vec<ConfigPriority>,
    migrator: ConfigMigrator,
}

impl std::fmt::Debug for UnifiedConfigCenter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UnifiedConfigCenter")
            .field("provider_count", &self.providers.len())
            .field("priorities", &self.priorities)
            .finish()
    }
}

impl Default for UnifiedConfigCenter {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedConfigCenter {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            priorities: vec![
                ConfigPriority::Default,
                ConfigPriority::File,
                ConfigPriority::Memory,
                ConfigPriority::Environment,
                ConfigPriority::Cli,
            ],
            migrator: ConfigMigrator::default(),
        }
    }

    pub fn with_provider(mut self, provider: impl ConfigProvider + 'static) -> Self {
        self.providers.push(Box::new(provider));
        self
    }

    pub fn with_migrator(mut self, migrator: ConfigMigrator) -> Self {
        self.migrator = migrator;
        self
    }

    pub fn load_config(&self) -> Result<AppConfig, ConfigError> {
        let mut documents = Vec::with_capacity(self.providers.len());
        for provider in &self.providers {
            documents.push(provider.load()?);
        }
        self.load_documents(documents)
    }

    pub fn load_value(&self, overlay: Value) -> Result<AppConfig, ConfigError> {
        self.load_documents(vec![ConfigDocument {
            source: ConfigSource::Memory {
                name: "import".to_string(),
            },
            priority: ConfigPriority::Memory,
            value: overlay,
        }])
    }

    pub fn load_documents(
        &self,
        mut documents: Vec<ConfigDocument>,
    ) -> Result<AppConfig, ConfigError> {
        documents.sort_by_key(|document| document.priority);

        let mut merged =
            serde_json::to_value(AppConfig::default()).map_err(|source| ConfigError::Parse {
                source_name: "defaults".to_string(),
                message: source.to_string(),
            })?;

        for document in documents {
            merge_value(&mut merged, document.value);
        }

        let migrated = self.migrator.migrate_to_current(merged)?;
        let config =
            serde_json::from_value::<AppConfig>(migrated).map_err(|source| ConfigError::Parse {
                source_name: "merged_config".to_string(),
                message: source.to_string(),
            })?;
        config.validate_startup()?;
        Ok(config)
    }

    fn providers_from_sources(
        &self,
        sources: &[ConfigSource],
    ) -> Result<Vec<Box<dyn ConfigProvider>>, ConfigError> {
        let mut providers: Vec<Box<dyn ConfigProvider>> = Vec::new();
        for source in sources {
            match source {
                ConfigSource::Default => {}
                ConfigSource::File(path) => {
                    providers.push(Box::new(FileConfigProvider::new(path.clone())))
                }
                ConfigSource::Environment { prefix } => {
                    providers.push(Box::new(EnvironmentConfigProvider::new(prefix.clone())))
                }
                ConfigSource::Memory { name } => {
                    return Err(ConfigError::SourceUnavailable {
                        source_name: format!("memory provider `{name}` requires an explicit value"),
                    })
                }
                ConfigSource::Cli => providers.push(Box::new(
                    MemoryConfigProvider::new("cli", Value::Object(Map::new()))
                        .with_priority(ConfigPriority::Cli),
                )),
            }
        }
        Ok(providers)
    }
}

impl ConfigCenter for UnifiedConfigCenter {
    fn load(&self, sources: &[ConfigSource]) -> Result<AppConfig, ConfigError> {
        let providers = self.providers_from_sources(sources)?;
        let mut documents = Vec::with_capacity(providers.len());
        for provider in providers {
            documents.push(provider.load()?);
        }
        self.load_documents(documents)
    }

    fn priorities(&self) -> &[ConfigPriority] {
        &self.priorities
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigSchema {
    pub schema_version: u32,
    pub precedence: Vec<ConfigPriority>,
    pub layers: Vec<ConfigLayerSchema>,
    pub default_config: AppConfig,
}

impl ConfigSchema {
    pub fn current() -> Self {
        Self {
            schema_version: CURRENT_CONFIG_VERSION,
            precedence: UnifiedConfigCenter::new().priorities().to_vec(),
            layers: vec![
                ConfigLayerSchema::new(
                    "global",
                    "Process-wide identity, environment, and filesystem roots",
                    vec![],
                ),
                ConfigLayerSchema::new(
                    "runtime",
                    "Runtime execution limits shared by TCP/HTTP/HTTPS adapters",
                    vec!["global"],
                ),
                ConfigLayerSchema::new("http", "HTTP listener and route defaults", vec!["runtime"]),
                ConfigLayerSchema::new(
                    "https",
                    "HTTPS listener and TLS-facing runtime defaults",
                    vec!["http", "tls", "domain"],
                ),
                ConfigLayerSchema::new(
                    "tls",
                    "Certificate source, renewal, and ACME challenge defaults",
                    vec!["global"],
                ),
                ConfigLayerSchema::new(
                    "domain",
                    "Domain validation, storage, and reservation policy",
                    vec!["global"],
                ),
                ConfigLayerSchema::new(
                    "dashboard",
                    "Monitoring dashboard refresh and retention defaults",
                    vec!["runtime"],
                ),
                ConfigLayerSchema::new(
                    "client",
                    "Desktop client preferences and connection defaults",
                    vec!["global"],
                ),
                ConfigLayerSchema::new("logging", "Shared logging defaults", vec!["global"]),
                ConfigLayerSchema::new("health", "Shared health-check defaults", vec!["runtime"]),
            ],
            default_config: AppConfig::default(),
        }
    }

    pub fn export_json(&self) -> Result<String, ConfigError> {
        serde_json::to_string_pretty(self).map_err(|source| ConfigError::Parse {
            source_name: "config_schema".to_string(),
            message: source.to_string(),
        })
    }

    pub fn default_config_json(&self) -> Result<String, ConfigError> {
        self.default_config.export_json()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigLayerSchema {
    pub name: String,
    pub description: String,
    pub depends_on: Vec<String>,
}

impl ConfigLayerSchema {
    fn new(name: &str, description: &str, depends_on: Vec<&str>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            depends_on: depends_on
                .into_iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        }
    }
}

pub trait ConfigMigration: Send + Sync {
    fn from_version(&self) -> u32;
    fn to_version(&self) -> u32;
    fn migrate(&self, value: Value) -> Result<Value, ConfigError>;
}

#[derive(Default)]
pub struct ConfigMigrator {
    migrations: Vec<Box<dyn ConfigMigration>>,
}

impl std::fmt::Debug for ConfigMigrator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ConfigMigrator")
            .field("migration_count", &self.migrations.len())
            .finish()
    }
}

impl ConfigMigrator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(mut self, migration: impl ConfigMigration + 'static) -> Self {
        self.migrations.push(Box::new(migration));
        self.migrations
            .sort_by_key(|migration| migration.from_version());
        self
    }

    pub fn migrate_to_current(&self, mut value: Value) -> Result<Value, ConfigError> {
        let mut version = config_version(&value).unwrap_or(0);
        if version == 0 {
            set_config_version(&mut value, CURRENT_CONFIG_VERSION);
            return Ok(value);
        }

        while version < CURRENT_CONFIG_VERSION {
            let Some(migration) = self
                .migrations
                .iter()
                .find(|migration| migration.from_version() == version)
            else {
                return Err(ConfigError::Migration {
                    from_version: version,
                    to_version: CURRENT_CONFIG_VERSION,
                    message: "missing migration step".to_string(),
                });
            };
            value = migration.migrate(value)?;
            version = migration.to_version();
            set_config_version(&mut value, version);
        }
        Ok(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigWatcher {
    interval: Duration,
}

impl Default for ConfigWatcher {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(2),
        }
    }
}

impl ConfigWatcher {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }

    pub fn watch_file<F>(
        &self,
        path: impl Into<PathBuf>,
        mut on_change: F,
    ) -> Result<ConfigWatchHandle, ConfigError>
    where
        F: FnMut(ConfigWatchEvent) + Send + 'static,
    {
        let path = path.into();
        if !path.exists() {
            return Err(ConfigError::SourceUnavailable {
                source_name: path.display().to_string(),
            });
        }

        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = Arc::clone(&stop);
        let interval = self.interval;
        let watched_path = path.clone();
        let mut last_modified = modified_at(&watched_path)?;
        let thread = thread::spawn(move || {
            while !stop_thread.load(Ordering::Relaxed) {
                thread::sleep(interval);
                let Ok(current_modified) = modified_at(&watched_path) else {
                    continue;
                };
                if current_modified > last_modified {
                    last_modified = current_modified;
                    on_change(ConfigWatchEvent {
                        path: watched_path.clone(),
                        changed_at: current_modified,
                    });
                }
            }
        });

        Ok(ConfigWatchHandle {
            stop,
            thread: Some(thread),
        })
    }

    pub fn watch_file_config<F>(
        &self,
        path: impl Into<PathBuf>,
        mut on_reload: F,
    ) -> Result<ConfigWatchHandle, ConfigError>
    where
        F: FnMut(Result<AppConfig, ConfigError>) + Send + 'static,
    {
        let path = path.into();
        self.watch_file(path.clone(), move |_| {
            let loaded = UnifiedConfigCenter::new()
                .with_provider(FileConfigProvider::new(path.clone()))
                .load_config();
            on_reload(loaded);
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigWatchEvent {
    pub path: PathBuf,
    pub changed_at: SystemTime,
}

#[derive(Debug)]
pub struct ConfigWatchHandle {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl ConfigWatchHandle {
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for ConfigWatchHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn merge_value(base: &mut Value, overlay: Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                merge_value(base_map.entry(key).or_insert(Value::Null), overlay_value);
            }
        }
        (base_slot, overlay_value) => {
            *base_slot = overlay_value;
        }
    }
}

fn set_path(root: &mut Value, path: &[String], value: Value) {
    let mut cursor = root;
    for segment in &path[..path.len().saturating_sub(1)] {
        let map = ensure_object(cursor);
        cursor = map
            .entry(segment.clone())
            .or_insert_with(|| Value::Object(Map::new()));
    }

    if let Some(last) = path.last() {
        ensure_object(cursor).insert(last.clone(), value);
    }
}

fn ensure_object(value: &mut Value) -> &mut Map<String, Value> {
    if !value.is_object() {
        *value = Value::Object(Map::new());
    }

    let Value::Object(map) = value else {
        *value = Value::Object(Map::new());
        return ensure_object(value);
    };
    map
}

fn parse_env_value(raw: &str) -> Value {
    if let Ok(value) = serde_json::from_str::<Value>(raw) {
        return value;
    }
    if let Ok(value) = raw.parse::<bool>() {
        return Value::Bool(value);
    }
    if let Ok(value) = raw.parse::<i64>() {
        return Value::Number(value.into());
    }
    if let Ok(value) = raw.parse::<u64>() {
        return Value::Number(value.into());
    }
    if let Ok(value) = raw.parse::<f64>() {
        if let Some(number) = serde_json::Number::from_f64(value) {
            return Value::Number(number);
        }
    }
    Value::String(raw.to_string())
}

fn validate_non_empty(key: &str, value: &str) -> Result<(), ConfigError> {
    if value.trim().is_empty() {
        validation_error(key, "must not be empty")
    } else {
        Ok(())
    }
}

fn validate_positive<T>(key: &str, value: T) -> Result<(), ConfigError>
where
    T: Default + PartialEq,
{
    if value == T::default() {
        validation_error(key, "must be greater than zero")
    } else {
        Ok(())
    }
}

fn validate_socket_addr(key: &str, value: &str) -> Result<(), ConfigError> {
    value
        .parse::<std::net::SocketAddr>()
        .map(|_| ())
        .map_err(|source| ConfigError::Validation {
            key: key.to_string(),
            message: source.to_string(),
        })
}

fn validation_error<T>(key: &str, message: &str) -> Result<T, ConfigError> {
    Err(ConfigError::Validation {
        key: key.to_string(),
        message: message.to_string(),
    })
}

fn modified_at(path: &Path) -> Result<SystemTime, ConfigError> {
    path.metadata()
        .and_then(|metadata| metadata.modified())
        .map_err(|source| ConfigError::SourceLoad {
            source_name: path.display().to_string(),
            message: source.to_string(),
        })
}

fn config_version(value: &Value) -> Option<u32> {
    value
        .get("version")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}

fn set_config_version(value: &mut Value, version: u32) {
    ensure_object(value).insert("version".to_string(), Value::Number(version.into()));
}
