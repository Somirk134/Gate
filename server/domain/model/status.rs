#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerifyStatus {
    Unverified,
    Pending,
    Verified,
    Failed(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindStatus {
    Unbound,
    Bound,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolveStatus {
    Unknown,
    Resolved,
    Unresolved,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DnsStatus {
    NotChecked,
    Matched,
    Mismatched,
    NoRecord,
    Error(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainStatus {
    Pending,
    Active,
    Disabled,
    Deleted,
}
