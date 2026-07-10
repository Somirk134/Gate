pub mod dns;
pub mod host;

pub use self::dns::{DnsAnswer, DnsCheckResult, DnsChecker, DnsQuery, DnsResolver};
#[cfg(test)]
pub use self::dns::TestDnsResolver;
pub use self::host::{HostResolver, RepositoryHostResolver};
