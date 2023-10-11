use eyre::Result;
use std::{path::Path, process::Command};

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
