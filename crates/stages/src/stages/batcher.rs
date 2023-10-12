use eyre::Result;
use op_primitives::{Artifacts, Monorepo};
use std::process::Command;
use std::rc::Rc;

/// Batcher Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Batcher {
    artifacts: Rc<Artifacts>,
    monorepo: Rc<Monorepo>,
}

impl crate::Stage for Batcher {
    /// Executes the [Batcher] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing batcher stage");

        // todo: this should be replaced with running the docker container inline through
        // the op-composer crate anyways so we won't need the docker directory at all.
        let proj_root = project_root::get_project_root()?;
        let docker_dir = proj_root.as_path().join("docker");

        let addresses_json = self.artifacts.l1_deployments();
        let addresses = crate::json::read_json(&addresses_json)?;

        let genesis_rollup_file = self.monorepo.genesis_rollup();
        let rollup_config = crate::json::read_json(&genesis_rollup_file)?;
        let start_batcher = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "batcher"])
            .env("PWD", &docker_dir)
            .env("L2OO_ADDRESS", addresses["L2OutputOracleProxy"].to_string())
            .env(
                "SEQUENCER_BATCH_INBOX_ADDRESS",
                rollup_config["batch_inbox_address"].to_string(),
            )
            .current_dir(docker_dir)
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
    pub fn new(artifacts: Rc<Artifacts>, monorepo: Rc<Monorepo>) -> Self {
        Self {
            artifacts,
            monorepo,
        }
    }
}
