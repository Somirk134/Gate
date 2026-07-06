use crate::model::{Domain, DomainId, Host};
use crate::storage::{DomainStorage, MemoryDomainStorage};

#[test]
fn memory_storage_insert_get_update_delete() -> Result<(), Box<dyn std::error::Error>> {
    let storage = MemoryDomainStorage::new();
    let id = DomainId::new("domain-1")?;
    let domain = Domain::builder(id.clone(), Host::new("api.gate.dev")?).build()?;

    storage.insert(domain.clone())?;
    assert!(storage.exists(domain.host())?);
    assert_eq!(storage.get(&id)?, Some(domain.clone()));

    let mut updated = domain.clone();
    updated.rename(Host::new("new.gate.dev")?);
    storage.update(updated.clone())?;
    assert_eq!(storage.find_by_host(updated.host())?, Some(updated.clone()));

    let deleted = storage.delete(&id)?;
    assert_eq!(deleted, Some(updated));
    assert_eq!(storage.get(&id)?, None);
    Ok(())
}
