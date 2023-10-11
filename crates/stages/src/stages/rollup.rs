use std::path::PathBuf;
use std::process::Command;

use eyre::Result;
use op_contracts::AddressManager;

/// Rollup Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Rollup {
    /// The path to the Dockerfile directory.
    pub docker_dir: Option<PathBuf>,
    /// The addresses.
    pub addresses: Option<AddressManager>,
    /// The rollup client choice.
    pub rollup_client: String,
}

impl crate::Stage for Rollup {
    /// Executes the [Rollup] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing rollup stage");

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

        let start_rollup = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "rollup-client"])
            .env("PWD", docker_dir)
            .env("ROLLUP_CLIENT_CHOICE", self.rollup_client)
            .current_dir(&docker_dir)
            .output()?;

        if !start_rollup.status.success() {
            eyre::bail!(
                "failed to start rollup client: {}",
                String::from_utf8_lossy(&start_rollup.stderr)
            );
        }

        crate::net::wait_up(op_config::ROLLUP_PORT, 30, 1)?;

        Ok(())
    }
}

impl Rollup {
    /// Creates a new stage.
    pub fn new(docker_dir: Option<PathBuf>, rollup_client: String) -> Self {
        Self {
            docker_dir: Some(docker_dir.unwrap_or(Rollup::get_docker_dir_unsafe())),
            rollup_client,
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
