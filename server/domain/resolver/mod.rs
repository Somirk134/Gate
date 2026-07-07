pub mod dns;
pub mod host;

pub use self::dns::{
    DnsAnswer, DnsCheckResult, DnsChecker, DnsQuery, DnsResolver, MockDnsResolver,
};
pub use self::host::{HostResolver, RepositoryHostResolver};
