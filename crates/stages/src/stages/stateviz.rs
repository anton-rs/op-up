use std::path::PathBuf;
use std::process::Command;

use eyre::Result;
use op_contracts::AddressManager;

/// Stateviz
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Stateviz {
    /// The path to the Dockerfile directory.
    pub docker_dir: Option<PathBuf>,
    /// The addresses.
    pub addresses: Option<AddressManager>,
}

impl crate::Stage for Stateviz {
    /// Executes the [Stateviz] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing stateviz stage");

        let docker_dir = self
            .docker_dir
            .as_ref()
            .map(|p| p.to_str())
            .flatten()
            .ok_or(eyre::eyre!("stateviz stage missing dockerfile directory"))?;

        // let addresses = self
        //     .addresses
        //     .as_ref()
        //     .ok_or(eyre::eyre!("stateviz stage missing addresses"))?;

        let proj_root = project_root::get_project_root()?;
        let addresses_json_file = proj_root.as_path().join(".devnet").join("addresses.json");
        let addresses = crate::json::read_json(&addresses_json_file)?;

        let start_stateviz = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "stateviz"])
            .env("PWD", docker_dir)
            .env("L2OO_ADDRESS", addresses["L2OutputOracleProxy"].to_string())
            .current_dir(&docker_dir)
            .output()?;

        // Check the output of the command.
        if !start_stateviz.status.success() {
            eyre::bail!(
                "stateviz failed to start: {}",
                String::from_utf8_lossy(&start_stateviz.stderr)
            );
        }

        Ok(())
    }
}

impl Stateviz {
    /// Creates a new stateviz stage,.
    pub fn new(docker_dir: Option<PathBuf>) -> Self {
        Self {
            docker_dir: Some(docker_dir.unwrap_or(Stateviz::get_docker_dir_unsafe())),
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
