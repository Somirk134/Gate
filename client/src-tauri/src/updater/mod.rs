use anyhow::{anyhow, Result};

pub async fn check_for_updates() -> Result<Option<String>> {
    Ok(None)
}

pub async fn install_update() -> Result<()> {
    Err(anyhow!("UPDATE_INSTALL_SERVICE_UNAVAILABLE"))
}
