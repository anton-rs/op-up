use std::path::PathBuf;
use std::process::Command;

use eyre::Result;

/// Cannon Prestate Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Prestate {
    /// The path to the monorepo.
    pub monorepo: Option<PathBuf>,
    /// The l2 genesis file.
    pub l2_genesis_file: Option<PathBuf>,
}

impl crate::Stage for Prestate {
    /// Executes the cannon prestate stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing cannon prestate stage");

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

        let op_program_bin = monorepo.join("op-program/bin");
        if !std::fs::metadata(op_program_bin).is_err() {
            tracing::info!(target: "stages", "cannon prestate already generated");
            return Ok(());
        }

        let make = Command::new("make")
            .args(["cannon-prestate"])
            .current_dir(&monorepo)
            .output()?;

        if !make.status.success() {
            eyre::bail!(
                "failed to generate cannon prestate: {}",
                String::from_utf8_lossy(&make.stderr)
            );
        }

        Ok(())
    }
}

impl Prestate {
    /// Creates a new prestate stage.
    pub fn new(monorepo: Option<PathBuf>, l2_genesis_file: Option<PathBuf>) -> Self {
        Self {
            monorepo: Some(monorepo.unwrap_or(Prestate::get_monorepo_dir_unsafe())),
            l2_genesis_file: Some(l2_genesis_file.unwrap_or(Prestate::get_l2_genesis_file_unsafe())),
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
