use std::path::PathBuf;
use std::process::Command;

use eyre::Result;
use op_contracts::AddressManager;

/// Layer 2 Execution Client Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Executor {
    /// The path to the Dockerfile directory.
    pub docker_dir: Option<PathBuf>,
    /// The addresses.
    pub addresses: Option<AddressManager>,
    /// The l2 client choice.
    pub l2_client: String,
}

impl crate::Stage for Executor {
    /// Executes the L2 Executor Stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing l2 execution client stage");

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

        let start_l2 = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "l2"])
            .env("PWD", docker_dir)
            .env("L2_CLIENT_CHOICE", &self.l2_client)
            .current_dir(&docker_dir)
            .output()?;

        if !start_l2.status.success() {
            eyre::bail!(
                "failed to start l2 execution client: {}",
                String::from_utf8_lossy(&start_l2.stderr)
            );
        }

        // todo: use a configured port here
        crate::net::wait_up(op_config::L2_PORT, 10, 1)?;

        Ok(())
    }
}

impl Executor {
    /// Creates a new stage.
    pub fn new(docker_dir: Option<PathBuf>, l2_client: String) -> Self {
        Self {
            docker_dir: Some(docker_dir.unwrap_or(Executor::get_docker_dir_unsafe())),
            l2_client,
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
