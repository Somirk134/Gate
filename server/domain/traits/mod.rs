use crate::error::DomainError;
use crate::model::{Domain, Host};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificateAuthorityKind {
    LetsEncryptReserved,
    ZeroSslReserved,
    Custom(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DnsProviderKind {
    CloudflareReserved,
    AliDnsReserved,
    TencentCloudDnsReserved,
    Route53Reserved,
    Custom(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateReservation {
    pub host: Host,
    pub authority: CertificateAuthorityKind,
}

pub trait HttpsDomainRuntimePort: Send + Sync {
    fn attach_domain(&self, domain: &Domain) -> Result<(), DomainError>;
    fn detach_domain(&self, domain: &Domain) -> Result<(), DomainError>;
}

pub trait TlsInfrastructurePort: Send + Sync {
    fn ensure_tls_ready(&self, domain: &Domain) -> Result<(), DomainError>;
}

pub trait CertificateProviderPort: Send + Sync {
    fn reserve_certificate(&self, host: &Host) -> Result<CertificateReservation, DomainError>;
}

pub trait AcmeClientPort: Send + Sync {
    fn authority(&self) -> CertificateAuthorityKind;
    fn prepare_challenge(&self, domain: &Domain) -> Result<(), DomainError>;
}

pub trait DnsProviderPort: Send + Sync {
    fn provider(&self) -> DnsProviderKind;
    fn prepare_domain_record(&self, domain: &Domain) -> Result<(), DomainError>;
}
