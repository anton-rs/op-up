use eyre::Result;
use std::process::Command;

/// Stateviz
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Stateviz;

impl crate::Stage for Stateviz {
    /// Executes the [Stateviz] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing stateviz stage");

        // todo: this should be replaced with running the docker container inline through
        // the op-composer crate anyways so we won't need the docker directory at all.
        let proj_root = project_root::get_project_root()?;
        let docker_dir = proj_root.as_path().join("docker");

        // todo: fix this to use the stack config artifacts dir instead of .devnet
        let addresses_json_file = proj_root.as_path().join(".devnet").join("addresses.json");
        let addresses = crate::json::read_json(&addresses_json_file)?;

        let start_stateviz = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "stateviz"])
            .env("PWD", &docker_dir)
            .env("L2OO_ADDRESS", addresses["L2OutputOracleProxy"].to_string())
            .current_dir(docker_dir)
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
    pub fn new() -> Self {
        Self
    }
}
