#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerifyStatus {
    Unverified,
    Pending,
    Verified,
    Failed(String),
}

impl VerifyStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Unverified => "UNVERIFIED",
            Self::Pending => "PENDING",
            Self::Verified => "VERIFIED",
            Self::Failed(_) => "FAILED",
        }
    }

    pub fn detail(&self) -> Option<&str> {
        match self {
            Self::Failed(message) => Some(message.as_str()),
            _ => None,
        }
    }

    pub fn from_parts(status: &str, detail: Option<String>) -> Option<Self> {
        match status {
            "UNVERIFIED" => Some(Self::Unverified),
            "PENDING" => Some(Self::Pending),
            "VERIFIED" => Some(Self::Verified),
            "FAILED" => Some(Self::Failed(detail.unwrap_or_default())),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindStatus {
    Unbound,
    Bound,
}

impl BindStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unbound => "UNBOUND",
            Self::Bound => "BOUND",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolveStatus {
    Unknown,
    Resolved,
    Unresolved,
}

impl ResolveStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "UNKNOWN",
            Self::Resolved => "RESOLVED",
            Self::Unresolved => "UNRESOLVED",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "UNKNOWN" => Some(Self::Unknown),
            "RESOLVED" => Some(Self::Resolved),
            "UNRESOLVED" => Some(Self::Unresolved),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DnsStatus {
    NotChecked,
    Matched,
    Mismatched,
    NoRecord,
    Error(String),
}

impl DnsStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::NotChecked => "NOT_CHECKED",
            Self::Matched => "MATCHED",
            Self::Mismatched => "MISMATCHED",
            Self::NoRecord => "NO_RECORD",
            Self::Error(_) => "ERROR",
        }
    }

    pub fn detail(&self) -> Option<&str> {
        match self {
            Self::Error(message) => Some(message.as_str()),
            _ => None,
        }
    }

    pub fn from_parts(status: &str, detail: Option<String>) -> Option<Self> {
        match status {
            "NOT_CHECKED" => Some(Self::NotChecked),
            "MATCHED" => Some(Self::Matched),
            "MISMATCHED" => Some(Self::Mismatched),
            "NO_RECORD" => Some(Self::NoRecord),
            "ERROR" => Some(Self::Error(detail.unwrap_or_default())),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainStatus {
    Pending,
    Active,
    Disabled,
    Deleted,
}

impl DomainStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Active => "ACTIVE",
            Self::Disabled => "DISABLED",
            Self::Deleted => "DELETED",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "PENDING" => Some(Self::Pending),
            "ACTIVE" => Some(Self::Active),
            "DISABLED" => Some(Self::Disabled),
            "DELETED" => Some(Self::Deleted),
            _ => None,
        }
    }
}
