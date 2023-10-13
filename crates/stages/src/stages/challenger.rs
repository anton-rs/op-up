use async_trait::async_trait;
use eyre::Result;
use op_primitives::Artifacts;
use std::process::Command;
use std::sync::Arc;

/// Challenger Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Challenger {
    artifacts: Arc<Artifacts>,
    challenger: String,
}

#[async_trait]
impl crate::Stage for Challenger {
    /// Executes the [Challenger] stage.
    async fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing challenger stage");

        // todo: this should be replaced with running the docker container inline through
        // the op-composer crate anyways so we won't need the docker directory at all.
        let proj_root = project_root::get_project_root()?;
        let docker_dir = proj_root.as_path().join("docker");

        let addresses_json = self.artifacts.l1_deployments();
        let addresses = crate::json::read_json(&addresses_json)?;

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
    pub fn new(artifacts: Arc<Artifacts>, challenger: String) -> Self {
        Self {
            artifacts,
            challenger,
        }
    }
}
