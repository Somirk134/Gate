use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::VersionError;

/// Semantic protocol version used by every header and registry entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl ProtocolVersion {
    pub const V1: Self = Self::new(1, 0, 0);
    pub const V2: Self = Self::new(2, 0, 0);

    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn is_compatible_with(self, other: Self) -> bool {
        self.major == other.major && self.major != 0
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::V1
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Policy used when client and server exchange supported versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionPolicy {
    Exact,
    SameMajor,
    LatestCompatible,
}

impl Default for VersionPolicy {
    fn default() -> Self {
        Self::LatestCompatible
    }
}

/// Version negotiation request and response model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionNegotiation {
    pub preferred: ProtocolVersion,
    pub supported: Vec<ProtocolVersion>,
    pub policy: VersionPolicy,
}

impl VersionNegotiation {
    pub fn v1() -> Self {
        Self {
            preferred: ProtocolVersion::V1,
            supported: vec![ProtocolVersion::V1],
            policy: VersionPolicy::LatestCompatible,
        }
    }

    pub fn negotiate(&self, remote: &[ProtocolVersion]) -> Result<ProtocolVersion, VersionError> {
        let mut candidates: Vec<_> = self
            .supported
            .iter()
            .copied()
            .filter(|local| remote.iter().any(|remote| self.accepts(*local, *remote)))
            .collect();

        candidates.sort();
        candidates
            .pop()
            .ok_or_else(|| VersionError::UnsupportedVersion {
                requested: self.preferred,
            })
    }

    fn accepts(&self, local: ProtocolVersion, remote: ProtocolVersion) -> bool {
        match self.policy {
            VersionPolicy::Exact => local == remote,
            VersionPolicy::SameMajor | VersionPolicy::LatestCompatible => {
                local.is_compatible_with(remote)
            }
        }
    }
}
