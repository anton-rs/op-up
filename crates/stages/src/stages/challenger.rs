use std::path::PathBuf;
use std::process::Command;

use eyre::Result;
use op_contracts::AddressManager;

/// Challenger Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Challenger {
    /// The path to the Dockerfile directory.
    pub docker_dir: Option<PathBuf>,
    /// The addresses.
    pub addresses: Option<AddressManager>,
    /// The challenger choice.
    pub challenger: String,
}

impl crate::Stage for Challenger {
    /// Executes the [Challenger] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing challenger stage");

        let docker_dir = self
            .docker_dir
            .as_ref()
            .and_then(|p| p.to_str())
            .ok_or(eyre::eyre!("challenger stage missing dockerfile directory"))?;

        // let addresses = self
        //     .addresses
        //     .as_ref()
        //     .ok_or(eyre::eyre!("challenger stage missing addresses"))?;

        let proj_root = project_root::get_project_root()?;
        let addresses_json_file = proj_root.as_path().join(".devnet").join("addresses.json");
        let addresses = crate::json::read_json(&addresses_json_file)?;

        let start_challenger = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "challenger"])
            .env("PWD", docker_dir)
            .env("L2OO_ADDRESS", addresses["L2OutputOracleProxy"].to_string())
            .env("DGF_ADDRESS", addresses["DisputeGameFactory"].to_string())
            .env("CHALLENGER_AGENT_CHOICE", &self.challenger)
            .current_dir(docker_dir)
            .output()?;

        // Check the output of the command.
        if !start_challenger.status.success() {
            eyre::bail!(
                "challenger failed to start: {}",
                String::from_utf8_lossy(&start_challenger.stderr)
            );
        }

        Ok(())
    }
}

impl Challenger {
    /// Creates a new challenger stage.
    pub fn new(docker_dir: Option<PathBuf>, challenger: String) -> Self {
        Self {
            docker_dir: Some(docker_dir.unwrap_or(Challenger::get_docker_dir_unsafe())),
            addresses: None,
            challenger,
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
