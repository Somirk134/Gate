use std::collections::HashMap;
#[cfg(feature = "sqlite")]
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::error::StorageError;
#[cfg(feature = "sqlite")]
use crate::model::{
    Alias, DnsStatus, DomainStatus, RecordType, ResolveStatus, TunnelId, VerifyStatus,
};
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

#[cfg(feature = "sqlite")]
#[derive(Clone, Debug)]
pub struct SqliteDomainStorage {
    path: Arc<PathBuf>,
}

#[cfg(feature = "sqlite")]
impl SqliteDomainStorage {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let storage = Self {
            path: Arc::new(path.into()),
        };
        storage.initialize()?;
        Ok(storage)
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn initialize(&self) -> Result<(), StorageError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| StorageError::Unavailable(error.to_string()))?;
        }

        let connection = self.connection()?;
        connection
            .execute_batch(
                r#"
                PRAGMA foreign_keys = ON;
                CREATE TABLE IF NOT EXISTS domains (
                    id TEXT PRIMARY KEY,
                    host TEXT NOT NULL UNIQUE,
                    tunnel_id TEXT,
                    record_type TEXT NOT NULL,
                    verify_status TEXT NOT NULL,
                    verify_detail TEXT,
                    resolve_status TEXT NOT NULL,
                    dns_status TEXT NOT NULL,
                    dns_detail TEXT,
                    status TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS domain_aliases (
                    domain_id TEXT NOT NULL,
                    alias TEXT NOT NULL UNIQUE,
                    PRIMARY KEY (domain_id, alias),
                    FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE
                );
                CREATE INDEX IF NOT EXISTS idx_domains_tunnel_id ON domains(tunnel_id);
                "#,
            )
            .map_err(sqlite_error)?;
        Ok(())
    }

    fn connection(&self) -> Result<rusqlite::Connection, StorageError> {
        rusqlite::Connection::open(self.path.as_path()).map_err(sqlite_error)
    }

    fn upsert_domain(
        &self,
        domain: Domain,
        insert_only: bool,
        require_existing: bool,
    ) -> Result<(), StorageError> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction().map_err(sqlite_error)?;

        if require_existing && !domain_exists(&transaction, domain.id())? {
            return Err(StorageError::NotFound(domain.id().to_string()));
        }

        if insert_only && domain_exists(&transaction, domain.id())? {
            return Err(StorageError::DuplicateKey(domain.id().to_string()));
        }

        let statement = if insert_only {
            r#"
            INSERT INTO domains (
                id, host, tunnel_id, record_type, verify_status, verify_detail,
                resolve_status, dns_status, dns_detail, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#
        } else {
            r#"
            INSERT INTO domains (
                id, host, tunnel_id, record_type, verify_status, verify_detail,
                resolve_status, dns_status, dns_detail, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(id) DO UPDATE SET
                host = excluded.host,
                tunnel_id = excluded.tunnel_id,
                record_type = excluded.record_type,
                verify_status = excluded.verify_status,
                verify_detail = excluded.verify_detail,
                resolve_status = excluded.resolve_status,
                dns_status = excluded.dns_status,
                dns_detail = excluded.dns_detail,
                status = excluded.status
            "#
        };

        let result = transaction.execute(
            statement,
            rusqlite::params![
                domain.id().as_str(),
                domain.host().as_str(),
                domain.tunnel_id().map(TunnelId::as_str),
                domain.record_type().as_str(),
                domain.verify_status().as_str(),
                domain.verify_status().detail(),
                domain.resolve_status().as_str(),
                domain.dns_status().as_str(),
                domain.dns_status().detail(),
                domain.status().as_str(),
            ],
        );

        match result {
            Ok(_) => {}
            Err(rusqlite::Error::SqliteFailure(error, _))
                if error.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                return Err(StorageError::DuplicateKey(domain.host().to_string()));
            }
            Err(error) => return Err(sqlite_error(error)),
        }

        transaction
            .execute(
                "DELETE FROM domain_aliases WHERE domain_id = ?1",
                rusqlite::params![domain.id().as_str()],
            )
            .map_err(sqlite_error)?;

        for alias in domain.aliases() {
            let result = transaction.execute(
                "INSERT INTO domain_aliases (domain_id, alias) VALUES (?1, ?2)",
                rusqlite::params![domain.id().as_str(), alias.as_str()],
            );
            match result {
                Ok(_) => {}
                Err(rusqlite::Error::SqliteFailure(error, _))
                    if error.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    return Err(StorageError::DuplicateKey(alias.to_string()));
                }
                Err(error) => return Err(sqlite_error(error)),
            }
        }

        transaction.commit().map_err(sqlite_error)?;
        Ok(())
    }

    fn load_domain(&self, id: &DomainId) -> Result<Option<Domain>, StorageError> {
        let connection = self.connection()?;
        load_domain_by_id(&connection, id)
    }
}

#[cfg(feature = "sqlite")]
impl DomainStorage for SqliteDomainStorage {
    fn insert(&self, domain: Domain) -> Result<(), StorageError> {
        self.upsert_domain(domain, true, false)
    }

    fn update(&self, domain: Domain) -> Result<(), StorageError> {
        self.upsert_domain(domain, false, true)
    }

    fn delete(&self, id: &DomainId) -> Result<Option<Domain>, StorageError> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction().map_err(sqlite_error)?;
        let domain = load_domain_by_id(&transaction, id)?;

        if domain.is_some() {
            transaction
                .execute(
                    "DELETE FROM domains WHERE id = ?1",
                    rusqlite::params![id.as_str()],
                )
                .map_err(sqlite_error)?;
        }

        transaction.commit().map_err(sqlite_error)?;
        Ok(domain)
    }

    fn get(&self, id: &DomainId) -> Result<Option<Domain>, StorageError> {
        self.load_domain(id)
    }

    fn find_by_host(&self, host: &Host) -> Result<Option<Domain>, StorageError> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                r#"
                SELECT id FROM domains WHERE host = ?1
                UNION
                SELECT domain_id FROM domain_aliases WHERE alias = ?1
                LIMIT 1
                "#,
            )
            .map_err(sqlite_error)?;
        let mut rows = statement
            .query(rusqlite::params![host.as_str()])
            .map_err(sqlite_error)?;

        let Some(row) = rows.next().map_err(sqlite_error)? else {
            return Ok(None);
        };

        let id = DomainId::new(row.get::<_, String>(0).map_err(sqlite_error)?)
            .map_err(|error| StorageError::Corrupted(error.to_string()))?;
        load_domain_by_id(&connection, &id)
    }

    fn list(&self) -> Result<Vec<Domain>, StorageError> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare("SELECT id FROM domains ORDER BY host ASC")
            .map_err(sqlite_error)?;
        let ids = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(sqlite_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(sqlite_error)?;

        let mut domains = Vec::with_capacity(ids.len());
        for id in ids {
            let id =
                DomainId::new(id).map_err(|error| StorageError::Corrupted(error.to_string()))?;
            if let Some(domain) = load_domain_by_id(&connection, &id)? {
                domains.push(domain);
            }
        }
        Ok(domains)
    }

    fn exists(&self, host: &Host) -> Result<bool, StorageError> {
        Ok(self.find_by_host(host)?.is_some())
    }
}

#[cfg(feature = "sqlite")]
fn domain_exists(connection: &rusqlite::Connection, id: &DomainId) -> Result<bool, StorageError> {
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM domains WHERE id = ?1",
            rusqlite::params![id.as_str()],
            |row| row.get(0),
        )
        .map_err(sqlite_error)?;
    Ok(count > 0)
}

#[cfg(feature = "sqlite")]
fn load_domain_by_id(
    connection: &rusqlite::Connection,
    id: &DomainId,
) -> Result<Option<Domain>, StorageError> {
    let mut statement = connection
        .prepare(
            r#"
            SELECT id, host, tunnel_id, record_type, verify_status, verify_detail,
                   resolve_status, dns_status, dns_detail, status
            FROM domains
            WHERE id = ?1
            "#,
        )
        .map_err(sqlite_error)?;
    let mut rows = statement
        .query(rusqlite::params![id.as_str()])
        .map_err(sqlite_error)?;

    let Some(row) = rows.next().map_err(sqlite_error)? else {
        return Ok(None);
    };

    let domain = row_to_domain(connection, row)?;
    Ok(Some(domain))
}

#[cfg(feature = "sqlite")]
fn row_to_domain(
    connection: &rusqlite::Connection,
    row: &rusqlite::Row<'_>,
) -> Result<Domain, StorageError> {
    let id = DomainId::new(row.get::<_, String>(0).map_err(sqlite_error)?)
        .map_err(|error| StorageError::Corrupted(error.to_string()))?;
    let host = Host::new(row.get::<_, String>(1).map_err(sqlite_error)?)
        .map_err(|error| StorageError::Corrupted(error.to_string()))?;
    let tunnel_id = row
        .get::<_, Option<String>>(2)
        .map_err(sqlite_error)?
        .map(TunnelId::new)
        .transpose()
        .map_err(|error| StorageError::Corrupted(error.to_string()))?;
    let record_type = RecordType::from_str(&row.get::<_, String>(3).map_err(sqlite_error)?)
        .ok_or_else(|| StorageError::Corrupted("invalid record type".to_string()))?;
    let verify_status = VerifyStatus::from_parts(
        &row.get::<_, String>(4).map_err(sqlite_error)?,
        row.get::<_, Option<String>>(5).map_err(sqlite_error)?,
    )
    .ok_or_else(|| StorageError::Corrupted("invalid verify status".to_string()))?;
    let resolve_status =
        ResolveStatus::from_str(&row.get::<_, String>(6).map_err(sqlite_error)?)
            .ok_or_else(|| StorageError::Corrupted("invalid resolve status".to_string()))?;
    let dns_status = DnsStatus::from_parts(
        &row.get::<_, String>(7).map_err(sqlite_error)?,
        row.get::<_, Option<String>>(8).map_err(sqlite_error)?,
    )
    .ok_or_else(|| StorageError::Corrupted("invalid dns status".to_string()))?;
    let status = DomainStatus::from_str(&row.get::<_, String>(9).map_err(sqlite_error)?)
        .ok_or_else(|| StorageError::Corrupted("invalid domain status".to_string()))?;
    let aliases = load_aliases(connection, &id)?;

    let mut builder = Domain::builder(id, host)
        .aliases(aliases)
        .record_type(record_type)
        .verify_status(verify_status)
        .resolve_status(resolve_status)
        .dns_status(dns_status)
        .status(status);

    if let Some(tunnel_id) = tunnel_id {
        builder = builder.tunnel_id(tunnel_id);
    }

    builder
        .build()
        .map_err(|error| StorageError::Corrupted(error.to_string()))
}

#[cfg(feature = "sqlite")]
fn load_aliases(
    connection: &rusqlite::Connection,
    id: &DomainId,
) -> Result<Vec<Alias>, StorageError> {
    let mut statement = connection
        .prepare("SELECT alias FROM domain_aliases WHERE domain_id = ?1 ORDER BY alias ASC")
        .map_err(sqlite_error)?;
    let aliases = statement
        .query_map(rusqlite::params![id.as_str()], |row| {
            row.get::<_, String>(0)
        })
        .map_err(sqlite_error)?
        .map(|value| {
            value.map_err(sqlite_error).and_then(|alias| {
                Alias::new(alias).map_err(|error| StorageError::Corrupted(error.to_string()))
            })
        })
        .collect();
    aliases
}

#[cfg(feature = "sqlite")]
fn sqlite_error(error: rusqlite::Error) -> StorageError {
    StorageError::Unavailable(error.to_string())
}

pub trait SqliteDomainStorageReserved: Send + Sync {}
pub trait RedisDomainStorageReserved: Send + Sync {}
pub trait JsonDomainStorageReserved: Send + Sync {}
pub trait FileDomainStorageReserved: Send + Sync {}
