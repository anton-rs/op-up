use eyre::Result;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Optimism monorepo git url.
pub const OP_MONOREPO_URL: &str = "git@github.com:ethereum-optimism/optimism.git";

/// The monorepo directory.
pub const MONOREPO_DIR: &str = "optimism";

/// The Optimism Monorepo.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Monorepo {
    /// Path for the directory holding the monorepo dir.
    pwd: PathBuf,
}

impl Monorepo {
    /// Creates a new monorepo instance.
    ///
    /// # Errors
    ///
    /// If the current working directory cannot be determined, this method will return an error.
    pub fn new() -> Result<Self> {
        Ok(Self {
            pwd: std::env::current_dir()?,
        })
    }

    /// Returns the path to the monorepo directory.
    pub fn path(&self) -> PathBuf {
        self.pwd.join(MONOREPO_DIR)
    }

    /// Returns the devnet artifacts directory.
    pub fn devnet(&self) -> PathBuf {
        self.path().join(".devnet")
    }

    /// Returns the L2 genesis file.
    pub fn l2_genesis_file(&self) -> PathBuf {
        self.devnet().join("l2-genesis.json")
    }
}

impl Monorepo {
    /// Clones the Optimism Monorepo into the given directory.
    pub fn git_clone(&self) -> Result<()> {
        tracing::info!(target: "monorepo", "Cloning optimism monorepo (this may take a while)...");
        git_clone(&self.pwd, OP_MONOREPO_URL)
    }
}

impl From<&Path> for Monorepo {
    fn from(local: &Path) -> Self {
        Self {
            pwd: local.to_path_buf(),
        }
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
