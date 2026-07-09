use super::CertificateStore;
use crate::certificate::{CertificateRecord, StoredCertificate};
use crate::error::CertificateError;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileCertificateStore {
    root: PathBuf,
}

impl FileCertificateStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn domain_dir(&self, domain: &str) -> PathBuf {
        self.root.join(sanitize_domain(domain))
    }

    fn metadata_path(&self, domain: &str) -> PathBuf {
        self.domain_dir(domain).join("metadata.json")
    }

    fn certificate_path(&self, domain: &str) -> PathBuf {
        self.domain_dir(domain).join("certificate.pem")
    }

    fn private_key_path(&self, domain: &str) -> PathBuf {
        self.domain_dir(domain).join("private_key.pem")
    }

    fn save_metadata(&self, record: &CertificateRecord) -> Result<(), CertificateError> {
        let domain_dir = self.domain_dir(&record.domain);
        fs::create_dir_all(&domain_dir)?;
        set_directory_permissions(&domain_dir)?;
        fs::write(
            self.metadata_path(&record.domain),
            serde_json::to_vec_pretty(record)?,
        )?;
        Ok(())
    }
}

impl CertificateStore for FileCertificateStore {
    fn save(&self, certificate: &StoredCertificate) -> Result<(), CertificateError> {
        let domain = &certificate.record.domain;
        let domain_dir = self.domain_dir(domain);
        fs::create_dir_all(&domain_dir)?;
        set_directory_permissions(&domain_dir)?;
        self.save_metadata(&certificate.record)?;
        fs::write(
            self.certificate_path(domain),
            certificate.certificate_pem.as_bytes(),
        )?;
        write_private_key(&self.private_key_path(domain), &certificate.private_key_pem)?;
        Ok(())
    }

    fn save_record(&self, record: &CertificateRecord) -> Result<(), CertificateError> {
        self.save_metadata(record)
    }

    fn load(&self, domain: &str) -> Result<StoredCertificate, CertificateError> {
        if !self.contains(domain)? {
            return Err(CertificateError::NotFound {
                domain: domain.to_string(),
            });
        }

        let record = serde_json::from_slice(&fs::read(self.metadata_path(domain))?)?;
        let certificate_pem = fs::read_to_string(self.certificate_path(domain))?;
        let private_key_pem = fs::read_to_string(self.private_key_path(domain))?;

        Ok(StoredCertificate {
            record,
            certificate_pem,
            private_key_pem,
        })
    }

    fn query(&self, domain: &str) -> Result<Option<CertificateRecord>, CertificateError> {
        if !self.metadata_path(domain).exists() {
            return Ok(None);
        }

        Ok(Some(serde_json::from_slice(&fs::read(
            self.metadata_path(domain),
        )?)?))
    }

    fn list(&self) -> Result<Vec<CertificateRecord>, CertificateError> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }

        let mut records: Vec<CertificateRecord> = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let metadata_path = entry.path().join("metadata.json");
            if metadata_path.exists() {
                records.push(serde_json::from_slice(&fs::read(metadata_path)?)?);
            }
        }
        records.sort_by(|left, right| left.domain.cmp(&right.domain));
        Ok(records)
    }

    fn delete(&self, domain: &str) -> Result<(), CertificateError> {
        let domain_dir = self.domain_dir(domain);
        if domain_dir.exists() {
            fs::remove_dir_all(domain_dir)?;
        }
        Ok(())
    }

    fn contains(&self, domain: &str) -> Result<bool, CertificateError> {
        Ok(self.metadata_path(domain).exists()
            && self.certificate_path(domain).exists()
            && self.private_key_path(domain).exists())
    }
}

fn write_private_key(path: &Path, private_key_pem: &str) -> Result<(), CertificateError> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)?;
    file.write_all(private_key_pem.as_bytes())?;
    file.sync_all()?;
    set_private_key_permissions(path)?;
    Ok(())
}

#[cfg(unix)]
fn set_directory_permissions(path: &Path) -> Result<(), CertificateError> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_directory_permissions(_path: &Path) -> Result<(), CertificateError> {
    Ok(())
}

#[cfg(unix)]
fn set_private_key_permissions(path: &Path) -> Result<(), CertificateError> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_private_key_permissions(_path: &Path) -> Result<(), CertificateError> {
    Ok(())
}

fn sanitize_domain(domain: &str) -> String {
    domain
        .chars()
        .map(|value| match value {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => value,
            _ => '_',
        })
        .collect()
}
