use eyre::Result;
use std::{path::Path, process::Command};

/// Optimism monorepo git url.
pub const OP_MONOREPO_URL: &str = "git@github.com:ethereum-optimism/optimism.git";

/// The monorepo directory.
pub const MONOREPO_DIR: &str = "optimism";

/// The Optimism Monorepo.
#[derive(Debug, Clone)]
pub struct Monorepo;

impl Monorepo {
    /// Clones the Optimism Monorepo into the given directory.
    pub fn clone(local: &Path) -> Result<()> {
        tracing::info!(target: "monorepo", "Cloning optimism monorepo (this may take a while)...");
        git_clone(local, OP_MONOREPO_URL)
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
