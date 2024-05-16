use anyhow::bail;
use anyhow::Context;

use super::Backend;
use super::VaultConfig;
use std::path::PathBuf;
use std::process::Command;
use std::{io::Write, process::Stdio};

#[derive(Debug, Clone)]
pub struct GoCryptFs {
    pub command_path: PathBuf,
    pub fusermount_path: PathBuf,
}

impl Default for GoCryptFs {
    fn default() -> Self {
        Self {
            command_path: PathBuf::from("gocryptfs"),
            fusermount_path: PathBuf::from("fusermount"),
        }
    }
}

impl GoCryptFs {
    fn code_to_err(code: i32) -> Option<anyhow::Error> {
        let msg = match code {
            0 => {
                return None;
            }
            6 => "The vault directory is not empty",
            10 => "The mount directory is not empty",
            12 => "The password is incorrect",
            22 => "The password is empty",
            23 => "Couldn't read configuration file",
            24 => "Couldn't write configuration file",
            26 => "Filesystem check reported an error",
            _ => "Unknown error",
        };

        Some(anyhow::anyhow!(msg))
    }
}

impl Backend for GoCryptFs {
    fn is_available(&self) -> bool {
        Command::new(&self.command_path)
            .arg("--version")
            .output()
            .map(|it| it.status.success())
            .unwrap_or_default()
    }

    fn create_vault(&self, cfg: &VaultConfig, password: &str) -> anyhow::Result<()> {
        let mut child = Command::new(&self.command_path)
            .stdin(Stdio::piped())
            .args(["--init", "-q", "--"])
            .arg(&cfg.vault_dir)
            .spawn()?;

        child
            .stdin
            .as_mut()
            .context("Failed to capture stdin")?
            .write_fmt(format_args!("{}", password))?;

        let status = child.wait()?;
        if !status.success() {
            bail!(
                GoCryptFs::code_to_err(status.code().context("Process didn't exit properly")?)
                    .expect("Must be an error due to non-success code")
            )
        }

        Ok(())
    }

    fn mount_vault(&self, cfg: &VaultConfig, password: &str) -> anyhow::Result<()> {
        let mut child = Command::new(&self.command_path)
            .stdin(Stdio::piped())
            .args(["-q", "--"])
            .arg(&cfg.vault_dir)
            .arg(&cfg.mount_dir)
            .spawn()?;

        child
            .stdin
            .as_mut()
            .context("Failed to capture stdin")?
            .write_fmt(format_args!("{}", password))?;

        let status = child.wait()?;
        if !status.success() {
            bail!(
                GoCryptFs::code_to_err(status.code().context("Process didn't exit properly")?)
                    .expect("Must be an error due to non-success code")
            );
        }

        Ok(())
    }

    fn close_vault(&self, cfg: &VaultConfig) -> anyhow::Result<()> {
        let status = Command::new(&self.fusermount_path)
            .arg("-u")
            .arg(&cfg.mount_dir)
            .output()?
            .status;

        if !status.success() {
            bail!(
                "Failed to close vault ({:?})",
                status.code().context("Process didn't exit properly")?
            );
        }

        Ok(())
    }
}
