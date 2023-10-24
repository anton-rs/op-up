use async_trait::async_trait;
use eyre::Result;
use op_primitives::{path_to_str, Artifacts, Monorepo};
use std::process::Command;
use std::sync::Arc;

/// L1 Genesis Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct L1Genesis {
    monorepo: Arc<Monorepo>,
    artifacts: Arc<Artifacts>,
    genesis_timestamp: u64,
}

#[async_trait]
impl crate::Stage for L1Genesis {
    /// Executes the [L1Genesis] stage.
    async fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing l1 genesis stage");

        // Artifacts paths
        let l1_genesis_artifact = self.artifacts.l1_genesis();
        let addresses_json_artifact = self.artifacts.l1_deployments();
        let addresses_json_artifact = path_to_str!(addresses_json_artifact)?;
        let jwt_secret_artifact = self.artifacts.jwt_secret();

        // Monorepo paths
        let deploy_config = self.monorepo.deploy_config();
        let deploy_config = path_to_str!(deploy_config)?;
        let allocs = self.monorepo.allocs();
        let allocs = path_to_str!(allocs)?;
        let op_node_dir = self.monorepo.op_node_dir();

        if !jwt_secret_artifact.exists() {
            tracing::info!(target: "stages", "Creating jwt secret...");
            // TODO: take this from the TOML stack config
            let jwt_secret = "688f5d737bad920bdfb2fc2f488d6b6209eebda1dae949a8de91398d932c517a";
            std::fs::write(&jwt_secret_artifact, jwt_secret)?;
        }

        if l1_genesis_artifact.exists() {
            tracing::info!(target: "stages", "L1 genesis already found.");
            return Ok(());
        }

        tracing::info!(target: "stages", "Creating L1 genesis...");
        let genesis_template =
            op_primitives::genesis::genesis_template_string(self.genesis_timestamp)
                .ok_or_else(|| eyre::eyre!("Could not create genesis template"))?;
        std::fs::write(&l1_genesis_artifact, genesis_template)?;
        let l1_genesis_artifact = path_to_str!(l1_genesis_artifact)?;
        let l1_genesis = Command::new("go")
            .args(["run", "cmd/main.go", "genesis", "l1"])
            .args(["--deploy-config", deploy_config])
            .args(["--l1-allocs", allocs])
            .args(["--l1-deployments", addresses_json_artifact])
            .args(["--outfile.l1", l1_genesis_artifact])
            .current_dir(op_node_dir)
            .output()?;

        if !l1_genesis.status.success() {
            eyre::bail!(
                "failed to create l1 genesis: {}",
                String::from_utf8_lossy(&l1_genesis.stderr)
            );
        }

        Ok(())
    }
}

impl L1Genesis {
    /// Creates a new stage.
    pub fn new(monorepo: Arc<Monorepo>, artifacts: Arc<Artifacts>, genesis_timestamp: u64) -> Self {
        Self {
            monorepo,
            artifacts,
            genesis_timestamp,
        }
    }
}
