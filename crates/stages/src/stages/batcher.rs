use std::path::PathBuf;
use std::process::Command;

use eyre::Result;
use op_contracts::AddressManager;

/// Batcher Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Batcher {
    /// The path to the Dockerfile directory.
    pub docker_dir: Option<PathBuf>,
    /// The genesis rollup file.
    pub genesis_rollup_file: Option<PathBuf>,
    /// The addresses.
    pub addresses: Option<AddressManager>,
}

impl crate::Stage for Batcher {
    /// Executes the [Batcher] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing batcher stage");

        let docker_dir = self
            .docker_dir
            .as_ref()
            .map(|p| p.to_str())
            .flatten()
            .ok_or(eyre::eyre!("missing dockerfile directory"))?;

        // let addresses = self
        //     .addresses
        //     .as_ref()
        //     .ok_or(eyre::eyre!("missing addresses"))?;

        let proj_root = project_root::get_project_root()?;
        let addresses_json_file = proj_root.as_path().join(".devnet").join("addresses.json");
        let addresses = crate::json::read_json(&addresses_json_file)?;

        let genesis_rollup_file = self
            .genesis_rollup_file
            .as_ref()
            .ok_or(eyre::eyre!("missing genesis rollup file"))?;

        let rollup_config = crate::json::read_json(&genesis_rollup_file)?;
        let start_batcher = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "batcher"])
            .env("PWD", docker_dir)
            .env("L2OO_ADDRESS", addresses["L2OutputOracleProxy"].to_string())
            .env(
                "SEQUENCER_BATCH_INBOX_ADDRESS",
                rollup_config["batch_inbox_address"].to_string(),
            )
            .current_dir(&docker_dir)
            .output()?;

        if !start_batcher.status.success() {
            eyre::bail!(
                "failed to start batcher: {}",
                String::from_utf8_lossy(&start_batcher.stderr)
            );
        }

        Ok(())
    }
}

impl Batcher {
    /// Creates a new stage.
    pub fn new(docker_dir: Option<PathBuf>, genesis_rollup_file: Option<PathBuf>) -> Self {
        Self {
            docker_dir: Some(docker_dir.unwrap_or(Batcher::get_docker_dir_unsafe())),
            genesis_rollup_file: Some(
                genesis_rollup_file.unwrap_or(Batcher::get_genesis_rollup_file()),
            ),
            addresses: None,
        }
    }

    /// Returns a [PathBuf] for the genesis rollup file.
    ///
    /// # Panics
    ///
    /// Panics if the [project_root::get_project_root] function call fails to return a valid
    /// project root [PathBuf].
    pub fn get_genesis_rollup_file() -> PathBuf {
        let proj_root = project_root::get_project_root().expect("Failed to get project root");
        let op_up_dir = proj_root.as_path();
        let devnet_dir = op_up_dir.join(".devnet");
        devnet_dir.join("rollup.json")
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
