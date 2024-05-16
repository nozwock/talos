pub mod cryfs;
pub mod gocryptfs;

use std::path::PathBuf;

pub trait Backend {
    fn is_available(&self) -> bool;
    fn create_vault(&self, cfg: &VaultConfig, password: &str) -> anyhow::Result<()>;
    fn mount_vault(&self, cfg: &VaultConfig, password: &str) -> anyhow::Result<()>;
    fn close_vault(&self, cfg: &VaultConfig) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct VaultConfig {
    pub vault_dir: PathBuf,
    pub mount_dir: PathBuf,
}
