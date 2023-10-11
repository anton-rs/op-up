use std::path::PathBuf;
use std::process::Command;

use eyre::Result;

/// Devnet Allocs Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Allocs {
    /// The path to the monorepo.
    pub monorepo: Option<PathBuf>,
    /// The l2 genesis file.
    pub l2_genesis_file: Option<PathBuf>,
}

impl crate::Stage for Allocs {
    /// Executes the allocs stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing allocs stage");

        let monorepo = self
            .monorepo
            .as_ref()
            .ok_or(eyre::eyre!("missing monorepo directory"))?;

        let l2_genesis_file = self
            .l2_genesis_file
            .as_ref()
            .map(|p| p.to_str())
            .flatten()
            .ok_or(eyre::eyre!("missing l2 genesis file"))?;

        if l2_genesis_file.exists() {
            tracing::info!(target: "stages", "l2 genesis file already found");
            return Ok(());
        }

        let allocs = Command::new("make")
            .args(["devnet-allocs"])
            .current_dir(&monorepo)
            .output()?;
        if !allocs.status.success() {
            eyre::bail!(
                "failed to generate devnet allocs: {}",
                String::from_utf8_lossy(&allocs.stderr)
            );
        }

        let copy_addr = Command::new("cp")
            .args([".devnet/addresses.json", "../.devnet/"])
            .current_dir(&monorepo)
            .output()?;
        if !copy_addr.status.success() {
            eyre::bail!(
                "failed to copy l1 deployments: {}",
                String::from_utf8_lossy(&copy_addr.stderr)
            );
        }

        let copy_allocs = Command::new("cp")
            .args([".devnet/allocs-l1.json", "../.devnet/"])
            .current_dir(&monorepo)
            .output()?;
        if !copy_allocs.status.success() {
            eyre::bail!(
                "failed to copy allocs: {}",
                String::from_utf8_lossy(&copy_allocs.stderr)
            );
        }

        Ok(())
    }
}

impl Allocs {
    /// Creates a new stage.
    pub fn new(monorepo: Option<PathBuf>, l2_genesis_file: Option<PathBuf>) -> Self {
        Self {
            monorepo: Some(monorepo.unwrap_or(Allocs::get_monorepo_dir_unsafe())),
            l2_genesis_file: Some(l2_genesis_file.unwrap_or(Allocs::get_l2_genesis_file_unsafe())),
        }
    }

    /// Returns a [PathBuf] for the monorepo directory.
    ///
    /// # Panics
    ///
    /// Panics if the [project_root::get_project_root] function call fails to return a valid
    /// project root [PathBuf].
    pub fn get_monorepo_dir_unsafe() -> PathBuf {
        let proj_root = project_root::get_project_root().expect("Failed to get project root");
        let op_up_dir = proj_root.as_path();
        op_up_dir.join("optimism")
    }

    /// Returns a [PathBuf] for the l2 genesis file.
    ///
    /// # Panics
    ///
    /// Panics if the [project_root::get_project_root] function call fails to return a valid
    /// project root [PathBuf].
    pub fn get_l2_genesis_file_unsafe() -> PathBuf {
        let proj_root = project_root::get_project_root().expect("Failed to get project root");
        let op_up_dir = proj_root.as_path();
        op_up_dir.join(".devnet").join("genesis-l2.json")
    }
}
