use async_trait::async_trait;
use eyre::Result;
use op_primitives::{path_to_str, Artifacts, Monorepo};
use std::process::Command;
use std::sync::Arc;

/// L2 Genesis Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct L2Genesis {
    l1_url: Option<String>,
    monorepo: Arc<Monorepo>,
    artifacts: Arc<Artifacts>,
}

#[async_trait]
impl crate::Stage for L2Genesis {
    /// Executes the [L2Genesis] stage.
    async fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing l2 genesis stage");

        // Artifacts paths
        let l2_genesis_artifact = self.artifacts.l2_genesis();
        let rollup_genesis_artifact = self.artifacts.rollup_genesis();
        let p2p_node_key_artifact = self.artifacts.p2p_node_key();

        // Monorepo paths
        let deploy_config = self.monorepo.deploy_config();
        let deploy_config = path_to_str!(deploy_config)?;
        // TODO: this should not be hardcoded to devnet but
        // the deployments dir should be chosen based on the network
        // from the stack.toml config.
        let devnet_deploys = self.monorepo.devnet_deploys();
        let devnet_deploys = path_to_str!(devnet_deploys)?;
        let op_node_dir = self.monorepo.op_node_dir();

        if !p2p_node_key_artifact.exists() {
            tracing::info!(target: "stages", "Creating p2p node key...");
            // TODO: take this from the TOML stack config
            let p2p_node_key = "dae4671006c60a3619556ace98eca6f6e092948d05b13070a27ac492a4fba419";
            std::fs::write(&p2p_node_key_artifact, p2p_node_key)?;
        }

        if l2_genesis_artifact.exists() && rollup_genesis_artifact.exists() {
            tracing::info!(target: "stages", "L2 and rollup genesis already found.");
            return Ok(());
        }

        let l2_genesis_str = path_to_str!(l2_genesis_artifact)?;
        let rollup_genesis_str = path_to_str!(rollup_genesis_artifact)?;

        tracing::info!(target: "stages", "Creating L2 and rollup genesis...");
        let l1_url = self.l1_url.clone().unwrap_or(op_config::L1_URL.to_owned());
        let l2_genesis = Command::new("go")
            .args(["run", "cmd/main.go", "genesis", "l2"])
            .args(["--l1-rpc", &l1_url])
            .args(["--deploy-config", deploy_config])
            .args(["--deployment-dir", devnet_deploys])
            .args(["--outfile.l2", l2_genesis_str])
            .args(["--outfile.rollup", rollup_genesis_str])
            .current_dir(op_node_dir)
            .output()?;

        if !l2_genesis.status.success() {
            eyre::bail!(
                "failed to create l2 genesis: {}",
                String::from_utf8_lossy(&l2_genesis.stderr)
            );
        }

        Ok(())
    }
}

impl L2Genesis {
    /// Creates a new stage.
    pub fn new(l1_url: Option<String>, monorepo: Arc<Monorepo>, artifacts: Arc<Artifacts>) -> Self {
        Self {
            l1_url,
            monorepo,
            artifacts,
        }
    }
}
