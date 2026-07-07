use crate::error::ResolveError;
use crate::model::{DomainStatus, Host, TunnelId};
use crate::repository::DomainRepository;

pub trait HostResolver: Send + Sync {
    fn resolve_host(&self, host: &str) -> Result<TunnelId, ResolveError>;
}

#[derive(Clone)]
pub struct RepositoryHostResolver<R>
where
    R: DomainRepository,
{
    repository: R,
}

impl<R> RepositoryHostResolver<R>
where
    R: DomainRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R> HostResolver for RepositoryHostResolver<R>
where
    R: DomainRepository,
{
    fn resolve_host(&self, host: &str) -> Result<TunnelId, ResolveError> {
        let host = Host::new(host).map_err(|error| ResolveError::InvalidHost(error.to_string()))?;
        let domain = self
            .repository
            .find_by_host(&host)
            .map_err(|error| {
                ResolveError::Storage(crate::error::StorageError::Unavailable(error.to_string()))
            })?
            .ok_or_else(|| ResolveError::HostNotFound(host.to_string()))?;

        if domain.status() == &DomainStatus::Disabled {
            return Err(ResolveError::DomainDisabled(domain.id().to_string()));
        }

        domain
            .tunnel_id()
            .cloned()
            .ok_or_else(|| ResolveError::UnboundHost(host.to_string()))
    }
}
