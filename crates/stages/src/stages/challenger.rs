use eyre::Result;
use std::process::Command;

/// Challenger Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Challenger {
    /// The challenger choice.
    pub challenger: String,
}

impl crate::Stage for Challenger {
    /// Executes the [Challenger] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing challenger stage");

        // todo: this should be replaced with running the docker container inline through
        // the op-composer crate anyways so we won't need the docker directory at all.
        let proj_root = project_root::get_project_root()?;
        let docker_dir = proj_root.as_path().join("docker");

        // todo: fix this to use the stack config artifacts dir instead of .devnet
        let addresses_json_file = proj_root.as_path().join(".devnet").join("addresses.json");
        let addresses = crate::json::read_json(&addresses_json_file)?;

        let start_challenger = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "challenger"])
            .env("PWD", &docker_dir)
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
    pub fn new(challenger: String) -> Self {
        Self { challenger }
    }
}
