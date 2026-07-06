use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
    pub auto_renew: bool,
    pub auto_apply: bool,
    pub preferred_chain: Option<String>,
    pub challenge_type: ChallengeType,
    pub dns_provider: Option<String>,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: None,
            key_path: None,
            auto_renew: true,
            auto_apply: false,
            preferred_chain: None,
            challenge_type: ChallengeType::Http01,
            dns_provider: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeType {
    #[default]
    Http01,
    Dns01,
    TlsAlpn01,
}
