use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

pub const PROTOCOL_VERSION: Version = Version {
    major: 0,
    minor: 1,
    patch: 0,
};
