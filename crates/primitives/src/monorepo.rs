use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// A macro to convert a [PathBuf] into a [Result<String>],
/// returning an error if the path cannot be converted to a string.
#[macro_export]
macro_rules! path_to_str {
    ($path:expr) => {
        $path
            .to_str()
            .ok_or_else(|| eyre::eyre!("Failed to convert path to string: {:?}", $path))
    };
}

/// Optimism Monorepo configuration.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MonorepoConfig {
    /// The name of the directory in which to store the Optimism Monorepo.
    pub directory_name: String,
    /// The source from which to obtain the Optimism Monorepo.
    pub source: MonorepoSource,
    /// The git URL from which to clone the Optimism Monorepo.
    pub git_url: String,
    /// The URL from which to download the Optimism Monorepo tarball.
    pub tarball_url: String,
    /// Optionally force overwriting local monorepo artifacts.
    pub force: bool,
}

impl Default for MonorepoConfig {
    fn default() -> Self {
        Self {
            source: MonorepoSource::Git,
            directory_name: "optimism".to_string(),
            git_url: "https://github.com/ethereum-optimism/optimism".to_string(),
            tarball_url: "https://github.com/ethereum-optimism/optimism/archive/develop.tar.gz"
                .to_string(),
            force: false,
        }
    }
}

/// The source from which to obtain the monorepo.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MonorepoSource {
    /// Clone from git.
    #[default]
    Git,
    /// Download from a tarball archive.
    Tarball,
}

/// The Optimism Monorepo.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Monorepo {
    /// Path for the directory holding the monorepo dir.
    pwd: PathBuf,
    /// Configuration for the monorepo.
    config: MonorepoConfig,
}

impl Monorepo {
    /// Creates a new monorepo instance with the given [MonorepoConfig] options.
    ///
    /// # Errors
    ///
    /// If the current working directory cannot be determined, this method will return an error.
    pub fn with_config(config: MonorepoConfig) -> Result<Self> {
        Ok(Self {
            pwd: std::env::current_dir()?,
            config,
        })
    }

    /// Returns the path to the monorepo directory.
    pub fn path(&self) -> PathBuf {
        self.pwd.join(&self.config.directory_name)
    }

    /// Returns the devnet artifacts directory.
    pub fn devnet(&self) -> PathBuf {
        self.path().join(".devnet")
    }

    /// Returns the L1 genesis file.
    pub fn l1_genesis(&self) -> PathBuf {
        self.devnet().join("genesis-l1.json")
    }

    /// Returns the L2 genesis file.
    pub fn l2_genesis(&self) -> PathBuf {
        self.devnet().join("genesis-l2.json")
    }

    /// Contracts directory.
    pub fn contracts(&self) -> PathBuf {
        self.path().join("packages/contracts-bedrock")
    }

    /// Deploy config file.
    pub fn deploy_config(&self) -> PathBuf {
        self.contracts().join("deploy-config/devnetL1.json")
    }

    /// Deployments directory.
    pub fn deployments(&self) -> PathBuf {
        self.contracts().join("deployments")
    }

    /// Devnet Deployments directory.
    pub fn devnet_deploys(&self) -> PathBuf {
        self.deployments().join("devnetL1")
    }

    /// Allocs file.
    pub fn allocs(&self) -> PathBuf {
        self.devnet().join("allocs-l1.json")
    }

    /// Addresses json file (the l1 deployments).
    pub fn addresses_json(&self) -> PathBuf {
        self.devnet().join("addresses.json")
    }

    /// Returns the op node directory.
    pub fn op_node_dir(&self) -> PathBuf {
        self.path().join("op-node")
    }

    /// Returns the genesis rollup file.
    pub fn genesis_rollup(&self) -> PathBuf {
        self.devnet().join("rollup.json")
    }
}

impl Monorepo {
    /// Obtains the monorepo from the given source.
    ///
    /// If the monorepo already exists, this method will garacefully log a warning and return.
    pub fn obtain_from_source(&self) -> Result<()> {
        if self.path().exists() && !self.config.force {
            tracing::warn!(target: "monorepo", "Monorepo already exists, skipping...");
            return Ok(());
        }

        match self.config.source {
            MonorepoSource::Git => self.git_clone(),
            MonorepoSource::Tarball => self.download(),
        }
    }

    /// Clones the Optimism Monorepo into the given directory.
    fn git_clone(&self) -> Result<()> {
        tracing::info!(target: "monorepo", "Cloning optimism monorepo (this may take a while)...");
        git_clone(&self.pwd, &self.config.git_url)
    }

    /// Downloads the Optimism Monorepo from the configured tarball archive.
    ///
    /// # Errors
    ///
    /// This function will return an Error if:
    /// - The archive cannot be downloaded
    /// - The downloaded file cannot be uncompressed
    /// - The resulting directory is not found or cannot be moved
    /// - The archive file cannot be deleted
    fn download(&self) -> Result<()> {
        tracing::info!(target: "monorepo", "Downloading optimism monorepo...");
        let archive_file_name = "optimism_monorepo.tar.gz";

        download_file(&self.pwd, &self.config.tarball_url, archive_file_name)?;
        unzip_tarball(&self.pwd, archive_file_name)?;
        mv_dir(&self.pwd, "optimism-develop", &self.config.directory_name)?;
        std::fs::remove_file(archive_file_name)?;
        Ok(())
    }
}

/// Clones a given git repository into the given directory.
pub(crate) fn git_clone(pwd: &Path, repo: &str) -> Result<()> {
    let out = Command::new("git")
        .arg("clone")
        .arg("--recursive")
        .arg("--depth")
        .arg("1")
        .arg(repo)
        .current_dir(pwd)
        .output()?;
    if !out.status.success() {
        eyre::bail!(
            "Failed to clone {} in {:?}: {}",
            repo,
            pwd,
            String::from_utf8_lossy(&out.stderr)
        )
    }

    Ok(())
}

/// Downloads a file from a given URL into the given directory.
pub(crate) fn download_file(pwd: &Path, url: &str, name: &str) -> Result<()> {
    let out = Command::new("curl")
        .arg("-L")
        .arg("--output")
        .arg(name)
        .arg(url)
        .current_dir(pwd)
        .output()?;
    if !out.status.success() {
        eyre::bail!(
            "Failed to download {} in {:?}: {}",
            url,
            pwd,
            String::from_utf8_lossy(&out.stderr)
        )
    }

    Ok(())
}

/// Unzips a tarball archive into the given directory.
pub(crate) fn unzip_tarball(pwd: &Path, name: &str) -> Result<()> {
    let out = Command::new("tar")
        .arg("-xvf")
        .arg(name)
        .current_dir(pwd)
        .output()?;
    if !out.status.success() {
        eyre::bail!(
            "Failed to unzip {} in {:?}: {}",
            name,
            pwd,
            String::from_utf8_lossy(&out.stderr)
        )
    }

    Ok(())
}

/// Moves a directory from one location to another.
pub(crate) fn mv_dir(pwd: &Path, src: &str, dst: &str) -> Result<()> {
    let out = Command::new("mv")
        .arg(src)
        .arg(dst)
        .current_dir(pwd)
        .output()?;
    if !out.status.success() {
        eyre::bail!(
            "Failed to move {} to {} in {:?}: {}",
            src,
            dst,
            pwd,
            String::from_utf8_lossy(&out.stderr)
        )
    }

    Ok(())
}
