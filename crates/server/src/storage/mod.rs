use anyhow::Result;
use std::path::PathBuf;

pub struct StorageManager {
    base_path: PathBuf,
}

impl StorageManager {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub fn get_data_dir(&self) -> &PathBuf {
        &self.base_path
    }

    pub fn ensure_directories(&self) -> Result<()> {
        std::fs::create_dir_all(&self.base_path)?;
        Ok(())
    }

    pub fn get_logs_dir(&self) -> PathBuf {
        self.base_path.join("logs")
    }

    pub fn get_db_path(&self) -> PathBuf {
        self.base_path.join("data").join("gate.db")
    }

    pub fn get_certs_dir(&self) -> PathBuf {
        self.base_path.join("certs")
    }
}
