use eyre::Result;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
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
/// assert!(DependencyManager::check_binary("cargo").is_some());
/// ```
#[derive(Debug, Clone, Default)]
pub struct DependencyManager;

impl DependencyManager {
    /// Default binaries to check for.
    pub const DEFAULT_BINARIES: &'static [&'static str] = &["docker", "curl", "tar", "make"];

    /// Linux package managers to check for.
    #[cfg(target_os = "linux")]
    pub const LINUX_PACKAGE_MANAGERS: &'static [(&'static str, &'static str)] =
        &[("apt", "install"), ("yum", "install"), ("pacman", "-S")];

    /// Checks for a package manager in the user's PATH.
    /// Returns the package manager name and the required argument to install a package.
    #[cfg(target_os = "linux")]
    pub fn package_manager() -> Option<(&'static str, &'static str)> {
        for (pm_name, pm_arg) in Self::LINUX_PACKAGE_MANAGERS {
            if Self::check_binary(*pm_name).is_some() {
                return Some((*pm_name, *pm_arg));
            }
        }
        None
    }

    /// Checks for a package manager in the user's PATH.
    /// This is a no-op on non-linux platforms.
    #[cfg(not(target_os = "linux"))]
    pub fn package_manager() -> Option<(&'static str, &'static str)> {
        None
    }

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
        tracing::info!("Installing Solidity version {}", version);
        let _ = svm_lib::install(&version).await?;
        svm_lib::use_version(&version)?;
        tracing::info!("Solidity version {} installed", version);
        Ok(())
    }

    /// Installs a binary.
    #[instrument(name = "deps", skip(binary))]
    pub fn install<P>(binary: P)
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        tracing::info!(target: "deps", "Installing {:?}", binary);

        if cfg!(target_os = "macos") {
            match Command::new("brew")
                .arg("install")
                .arg(binary.as_ref())
                .output()
            {
                Ok(out) => tracing::info!("Installed {:?} with output: {:?}", binary, out.status),
                Err(e) => tracing::warn!("Failed to install {:?} with err: {:?}", binary, e),
            }
        } else if cfg!(target_os = "linux") {
            let Some((pm_name, pm_arg)) = Self::package_manager() else {
                tracing::warn!("Failed to find package manager to install required dependencies");
                return;
            };
            match Command::new(pm_name)
                .arg(pm_arg)
                .arg(binary.as_ref())
                .output()
            {
                Ok(out) => tracing::info!("Installed {:?} with output: {:?}", binary, out.status),
                Err(e) => tracing::warn!("Failed to install {:?} with err: {:?}", binary, e),
            }
        } else {
            tracing::warn!(
                "Automatic install not supported for OS: {}",
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

        let mut curl_command = Command::new("curl");
        curl_command.arg("-L");
        curl_command.arg("https://foundry.paradigm.xyz");
        curl_command.stdout(std::process::Stdio::piped());

        let mut bash_command = Command::new("bash");
        let spawned = curl_command.spawn()?;
        let output = spawned
            .stdout
            .ok_or_else(|| eyre::eyre!("Failed to spawn curl"))?;
        bash_command.stdin(output);

        let output = bash_command.output()?;
        tracing::info!("Foundry installed with output: {}", output.status);

        let mut foundryup_command = Command::new("foundryup");
        let output = foundryup_command.output()?;
        tracing::info!("Foundryup executed with output: {}", output.status);

        Ok(())
    }

    /// Installs Go.
    #[instrument(name = "deps")]
    pub fn go() -> Result<()> {
        if Self::check_binary("go").is_some() {
            tracing::debug!("Go already installed");
            return Ok(());
        }
        tracing::info!("Installing Go");

        if cfg!(target_os = "macos") {
            match Command::new("brew").arg("install").arg("go").output() {
                Ok(out) => tracing::info!("Installed Go with output: {:?}", out.status),
                Err(e) => tracing::warn!("Failed to install Go with err: {:?}", e),
            }
        } else if cfg!(target_os = "linux") {
            let Some((pm_name, pm_arg)) = Self::package_manager() else {
                tracing::warn!("Failed to find package manager to install Go");
                return Err(eyre::eyre!("Failed to find package manager to install Go"));
            };
            match Command::new(pm_name).arg(pm_arg).arg("golang-go").output() {
                Ok(out) => tracing::info!("Installed Go with output: {:?}", out.status),
                Err(e) => tracing::warn!("Failed to install Go with err: {:?}", e),
            }
        } else {
            tracing::warn!(
                "Automatic install not supported for OS: {}",
                std::env::consts::OS
            );
        }

        Ok(())
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
        assert!(DependencyManager::check_binary("cargo").is_some());
    }
}
