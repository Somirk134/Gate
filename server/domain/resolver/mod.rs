pub mod dns;
pub mod host;

#[cfg(test)]
pub use self::dns::TestDnsResolver;
pub use self::dns::{DnsAnswer, DnsCheckResult, DnsChecker, DnsQuery, DnsResolver};
pub use self::host::{HostResolver, RepositoryHostResolver};
