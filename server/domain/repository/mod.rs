use crate::error::DomainError;
use crate::model::{Domain, DomainId, Host, TunnelId};
use crate::storage::{DomainStorage, MemoryDomainStorage};

pub type DomainRepositoryResult<T> = Result<T, DomainError>;

pub trait DomainRepository: Send + Sync {
    fn create(&self, domain: Domain) -> DomainRepositoryResult<Domain>;
    fn delete(&self, id: &DomainId) -> DomainRepositoryResult<Domain>;
    fn update(&self, domain: Domain) -> DomainRepositoryResult<Domain>;
    fn find_by_id(&self, id: &DomainId) -> DomainRepositoryResult<Option<Domain>>;
    fn find_by_host(&self, host: &Host) -> DomainRepositoryResult<Option<Domain>>;
    fn find_by_tunnel(&self, tunnel_id: &TunnelId) -> DomainRepositoryResult<Vec<Domain>>;
    fn list(&self) -> DomainRepositoryResult<Vec<Domain>>;
    fn exists(&self, host: &Host) -> DomainRepositoryResult<bool>;
    fn bind_tunnel(&self, id: &DomainId, tunnel_id: TunnelId) -> DomainRepositoryResult<Domain>;
    fn unbind_tunnel(&self, id: &DomainId) -> DomainRepositoryResult<Domain>;
}

#[derive(Clone, Default)]
pub struct MemoryRepository {
    storage: MemoryDomainStorage,
}

impl MemoryRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_storage(storage: MemoryDomainStorage) -> Self {
        Self { storage }
    }

    pub fn storage(&self) -> &MemoryDomainStorage {
        &self.storage
    }
}

impl DomainRepository for MemoryRepository {
    fn create(&self, domain: Domain) -> DomainRepositoryResult<Domain> {
        self.ensure_domain_hosts_available(&domain, None)?;

        self.storage.insert(domain.clone())?;
        Ok(domain)
    }

    fn delete(&self, id: &DomainId) -> DomainRepositoryResult<Domain> {
        self.storage
            .delete(id)?
            .ok_or_else(|| DomainError::NotFound(id.to_string()))
    }

    fn update(&self, domain: Domain) -> DomainRepositoryResult<Domain> {
        self.ensure_domain_hosts_available(&domain, Some(domain.id()))?;

        self.storage.update(domain.clone())?;
        Ok(domain)
    }

    fn find_by_id(&self, id: &DomainId) -> DomainRepositoryResult<Option<Domain>> {
        Ok(self.storage.get(id)?)
    }

    fn find_by_host(&self, host: &Host) -> DomainRepositoryResult<Option<Domain>> {
        Ok(self.storage.find_by_host(host)?)
    }

    fn find_by_tunnel(&self, tunnel_id: &TunnelId) -> DomainRepositoryResult<Vec<Domain>> {
        Ok(self
            .storage
            .list()?
            .into_iter()
            .filter(|domain| domain.tunnel_id() == Some(tunnel_id))
            .collect())
    }

    fn list(&self) -> DomainRepositoryResult<Vec<Domain>> {
        Ok(self.storage.list()?)
    }

    fn exists(&self, host: &Host) -> DomainRepositoryResult<bool> {
        Ok(self.storage.exists(host)?)
    }

    fn bind_tunnel(&self, id: &DomainId, tunnel_id: TunnelId) -> DomainRepositoryResult<Domain> {
        let mut domain = self
            .find_by_id(id)?
            .ok_or_else(|| DomainError::NotFound(id.to_string()))?;
        domain.bind(tunnel_id)?;
        self.update(domain)
    }

    fn unbind_tunnel(&self, id: &DomainId) -> DomainRepositoryResult<Domain> {
        let mut domain = self
            .find_by_id(id)?
            .ok_or_else(|| DomainError::NotFound(id.to_string()))?;
        domain.unbind()?;
        self.update(domain)
    }
}

impl MemoryRepository {
    fn ensure_domain_hosts_available(
        &self,
        domain: &Domain,
        current_id: Option<&DomainId>,
    ) -> DomainRepositoryResult<()> {
        self.ensure_host_available(domain.host(), current_id)?;
        for alias in domain.aliases() {
            self.ensure_host_available(alias.host(), current_id)?;
        }
        Ok(())
    }

    fn ensure_host_available(
        &self,
        host: &Host,
        current_id: Option<&DomainId>,
    ) -> DomainRepositoryResult<()> {
        if let Some(existing) = self.storage.find_by_host(host)? {
            if current_id != Some(existing.id()) {
                return Err(DomainError::AlreadyExists(host.to_string()));
            }
        }
        Ok(())
    }
}
