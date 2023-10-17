use eyre::Result;
use std::path::{Path, PathBuf};
use tracing::instrument;

/// DependencyManager
///
/// The DependencyManager is responsible for checking for the existence of
/// binaries in the user's PATH. For each binary that is not installed, it
/// will prompt the user with [inquire] to install the binary.
///
/// # Examples
///
/// ```rust
/// use opup::deps::DependencyManager;
///
/// assert_eq!(DependencyManager::check_binary("cargo").is_some(), true);
/// ```
#[derive(Debug, Clone, Default)]
pub struct DependencyManager;

impl DependencyManager {
    /// Default binaries to check for.
    pub const DEFAULT_BINARIES: &'static [&'static str] = &["docker", "curl", "tar"];

    /// Installs binaries that are not in the user's PATH.
    #[instrument(name = "deps")]
    pub async fn sync() -> Result<()> {
        for binary in Self::DEFAULT_BINARIES {
            tracing::debug!("Checking for {}", binary);
            if Self::check_binary(*binary).is_some() {
                continue;
            }
            tracing::warn!("{} not found", binary);
            match inquire::Confirm::new(&format!("Missing \"{}\" in path, install?", binary))
                .prompt()
                .ok()
            {
                Some(true) => DependencyManager::install(binary),
                // bail if the answer is no _or_ the user cancelled the prompt
                _ => eyre::bail!("Cannot proceed without \"{}\" installed.", binary),
            }
        }
        Self::solidity().await?;
        Self::foundry()
    }

    /// Installs solidity using [svm_lib].
    #[instrument(name = "deps")]
    pub async fn solidity() -> Result<()> {
        tracing::info!("SVM data dir: {:?}", svm_lib::SVM_DATA_DIR.to_path_buf());
        let installed = svm_lib::current_version().map(|o| o.is_some()).ok() == Some(true);
        if Self::check_binary("solc").is_some() && installed {
            tracing::debug!("solc already installed");
            return Ok(());
        }
        let version = semver::Version::parse("0.8.17")?;
        tracing::info!("Installing Solidity version {:?}", version);
        let _ = svm_lib::install(&version).await?;
        tracing::info!("Solidity version {:?} installed", version);
        return Ok(());
    }

    /// Installs a binary.
    #[instrument(name = "deps", skip(binary))]
    pub fn install<P>(binary: P)
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        tracing::info!(target: "deps", "Installing {:?}", binary);
        if cfg!(target_os = "macos") {
            let mut brew_command = std::process::Command::new("brew");
            brew_command.arg("install");
            brew_command.arg(binary.as_ref());
            match brew_command.output() {
                Ok(output) => {
                    tracing::info!("Installed {:?} with output: {}", binary, output.status)
                }
                Err(e) => tracing::warn!("Failed to install {:?} with err: {:?}", binary, e),
            }
        } else if cfg!(target_os = "linux") {
            let mut apt_command = std::process::Command::new("apt");
            apt_command.arg("install");
            apt_command.arg(binary.as_ref());
            match apt_command.output() {
                Ok(output) => {
                    tracing::info!("Installed {:?} with output: {}", binary, output.status)
                }
                Err(e) => tracing::warn!("Failed to install {:?} with err: {:?}", binary, e),
            }
        } else {
            tracing::warn!(
                "Automatic installed not supported for OS: {}",
                std::env::consts::OS
            );
        }
    }

    /// Installs foundry and all it's dependencies.
    #[instrument(name = "deps")]
    pub fn foundry() -> Result<()> {
        if Self::check_binary("forge").is_some() {
            tracing::debug!("foundry already installed");
            return Ok(());
        }
        tracing::info!("Installing Foundry");

        let mut curl_command = std::process::Command::new("curl");
        curl_command.arg("-L");
        curl_command.arg("https://foundry.paradigm.xyz");
        curl_command.stdout(std::process::Stdio::piped());

        let mut bash_command = std::process::Command::new("bash");
        let spawned = curl_command.spawn()?;
        let output = spawned
            .stdout
            .ok_or_else(|| eyre::eyre!("Failed to spawn curl"))?;
        bash_command.stdin(output);

        let output = bash_command.output()?;
        tracing::info!("Foundry installed with output: {}", output.status);

        let mut foundryup_command = std::process::Command::new("foundryup");
        let output = foundryup_command.output()?;
        tracing::info!("Foundryup executed with output: {}", output.status);

        return Ok(());
    }

    /// Checks for an individual binary in the user's PATH.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use opup::deps::DependencyManager;
    ///
    /// assert_eq!(DependencyManager::check_binary("cargo").is_some(), true);
    /// ```
    #[instrument(name = "deps", skip(exec_name))]
    pub fn check_binary<P>(exec_name: P) -> Option<PathBuf>
    where
        P: Into<String>,
    {
        which::which::<String>(exec_name.into()).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_binary() {
        assert_eq!(DependencyManager::check_binary("cargo").is_some(), true);
    }
}
