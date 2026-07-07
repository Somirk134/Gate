use anyhow::{anyhow, Result};

pub async fn check_for_updates() -> Result<Option<String>> {
    Ok(None)
}

pub async fn install_update() -> Result<()> {
    Err(anyhow!("该功能暂未实现：更新安装尚未接入真实更新服务。"))
}
