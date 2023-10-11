use std::path::PathBuf;
use std::process::Command;

use eyre::Result;
use op_contracts::AddressManager;

/// Proposer Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Proposer {
    /// The path to the Dockerfile directory.
    pub docker_dir: Option<PathBuf>,
    /// The addresses.
    pub addresses: Option<AddressManager>,
}

impl crate::Stage for Proposer {
    /// Executes the [Proposer] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing proposer stage");

        let docker_dir = self
            .docker_dir
            .as_ref()
            .map(|p| p.to_str())
            .flatten()
            .ok_or(eyre::eyre!("missing dockerfile directory"))?;

        let addresses = self
            .addresses
            .as_ref()
            .ok_or(eyre::eyre!("missing addresses"))?;

        let start_proposer = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "proposer"])
            .env("PWD", docker_dir)
            .env("L2OO_ADDRESS", addresses["L2OutputOracleProxy"].to_string())
            .current_dir(&docker_dir)
            .output()?;

        if !start_proposer.status.success() {
            eyre::bail!(
                "failed to start proposer: {}",
                String::from_utf8_lossy(&start_proposer.stderr)
            );
        }

        Ok(())
    }
}

impl Proposer {
    /// Creates a new stage.
    pub fn new(docker_dir: Option<PathBuf>) -> Self {
        Self {
            docker_dir: Some(docker_dir.unwrap_or(Proposer::get_docker_dir_unsafe())),
            addresses: None,
        }
    }

    /// Returns a [PathBuf] for the Dockerfile directory.
    ///
    /// # Panics
    ///
    /// Panics if the [project_root::get_project_root] function call fails to return a valid
    /// project root [PathBuf].
    pub fn get_docker_dir_unsafe() -> PathBuf {
        let proj_root = project_root::get_project_root().expect("Failed to get project root");
        let op_up_dir = proj_root.as_path();
        op_up_dir.join("docker")
    }
}