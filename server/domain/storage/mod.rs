use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::StorageError;
use crate::model::{Domain, DomainId, Host};

pub trait DomainStorage: Send + Sync {
    fn insert(&self, domain: Domain) -> Result<(), StorageError>;
    fn update(&self, domain: Domain) -> Result<(), StorageError>;
    fn delete(&self, id: &DomainId) -> Result<Option<Domain>, StorageError>;
    fn get(&self, id: &DomainId) -> Result<Option<Domain>, StorageError>;
    fn find_by_host(&self, host: &Host) -> Result<Option<Domain>, StorageError>;
    fn list(&self) -> Result<Vec<Domain>, StorageError>;
    fn exists(&self, host: &Host) -> Result<bool, StorageError>;
}

#[derive(Clone, Default)]
pub struct MemoryDomainStorage {
    inner: Arc<RwLock<HashMap<DomainId, Domain>>>,
}

impl MemoryDomainStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&self) -> Result<(), StorageError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|_| StorageError::LockPoisoned("memory-domain-storage"))?;
        guard.clear();
        Ok(())
    }
}

impl DomainStorage for MemoryDomainStorage {
    fn insert(&self, domain: Domain) -> Result<(), StorageError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|_| StorageError::LockPoisoned("memory-domain-storage"))?;

        if guard.contains_key(domain.id()) {
            return Err(StorageError::DuplicateKey(domain.id().to_string()));
        }

        guard.insert(domain.id().clone(), domain);
        Ok(())
    }

    fn update(&self, domain: Domain) -> Result<(), StorageError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|_| StorageError::LockPoisoned("memory-domain-storage"))?;

        if !guard.contains_key(domain.id()) {
            return Err(StorageError::NotFound(domain.id().to_string()));
        }

        guard.insert(domain.id().clone(), domain);
        Ok(())
    }

    fn delete(&self, id: &DomainId) -> Result<Option<Domain>, StorageError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|_| StorageError::LockPoisoned("memory-domain-storage"))?;
        Ok(guard.remove(id))
    }

    fn get(&self, id: &DomainId) -> Result<Option<Domain>, StorageError> {
        let guard = self
            .inner
            .read()
            .map_err(|_| StorageError::LockPoisoned("memory-domain-storage"))?;
        Ok(guard.get(id).cloned())
    }

    fn find_by_host(&self, host: &Host) -> Result<Option<Domain>, StorageError> {
        let guard = self
            .inner
            .read()
            .map_err(|_| StorageError::LockPoisoned("memory-domain-storage"))?;

        Ok(guard
            .values()
            .find(|domain| {
                domain.host() == host || domain.aliases().iter().any(|alias| alias.host() == host)
            })
            .cloned())
    }

    fn list(&self) -> Result<Vec<Domain>, StorageError> {
        let guard = self
            .inner
            .read()
            .map_err(|_| StorageError::LockPoisoned("memory-domain-storage"))?;
        Ok(guard.values().cloned().collect())
    }

    fn exists(&self, host: &Host) -> Result<bool, StorageError> {
        Ok(self.find_by_host(host)?.is_some())
    }
}

pub trait SqliteDomainStorageReserved: Send + Sync {}
pub trait RedisDomainStorageReserved: Send + Sync {}
pub trait JsonDomainStorageReserved: Send + Sync {}
pub trait FileDomainStorageReserved: Send + Sync {}
