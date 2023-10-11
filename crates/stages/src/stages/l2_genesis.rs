use std::path::PathBuf;
use std::process::Command;

use eyre::Result;
// use op_contracts::AddressManager;

/// L2 Genesis Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct L2Genesis {
    /// The path to the deployment config file.
    pub deploy_config_file: Option<PathBuf>,
    /// The deployment directory.
    pub deployment_dir: Option<PathBuf>,
    /// The l2 genesis file.
    pub l2_genesis_file: Option<PathBuf>,
    /// The genesis rollup file.
    pub genesis_rollup_file: Option<PathBuf>,
    /// The op node directory.
    pub op_node_dir: Option<PathBuf>,
}

impl crate::Stage for L2Genesis {
    /// Executes the [L2Genesis] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing l2 genesis stage");

        let deploy_config_file = self
            .deploy_config_file
            .as_ref()
            .and_then(|p| p.to_str())
            .ok_or(eyre::eyre!("missing deploy config file"))?;

        let deployment_dir = self
            .deployment_dir
            .as_ref()
            .and_then(|p| p.to_str())
            .ok_or(eyre::eyre!("missing deployment directory"))?;

        let l2_genesis_file = self
            .l2_genesis_file
            .as_ref()
            .ok_or(eyre::eyre!("missing l2 genesis file"))?;

        let genesis_rollup_file = self
            .genesis_rollup_file
            .as_ref()
            .and_then(|p| p.to_str())
            .ok_or(eyre::eyre!("missing genesis rollup file"))?;

        let op_node_dir = self
            .op_node_dir
            .as_ref()
            .and_then(|p| p.to_str())
            .ok_or(eyre::eyre!("missing op node directory"))?;

        if l2_genesis_file.exists() {
            tracing::info!(target: "stages", "L2 genesis already found.");
            return Ok(());
        }

        let l2_genesis_file_str = l2_genesis_file
            .to_str()
            .unwrap_or("failed to stringify l2 genesis file");

        tracing::info!(target: "stages", "Creating L2 and rollup genesis...");
        let l2_genesis = Command::new("go")
            .args(["run", "cmd/main.go", "genesis", "l2"])
            .args(["--l1-rpc", op_config::L1_URL])
            .args(["--deploy-config", deploy_config_file])
            .args(["--deployment-dir", deployment_dir])
            .args(["--outfile.l2", l2_genesis_file_str])
            .args(["--outfile.rollup", genesis_rollup_file])
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
    pub fn new(
        deploy_config_file: Option<PathBuf>,
        deployment_dir: Option<PathBuf>,
        l2_genesis_file: Option<PathBuf>,
        genesis_rollup_file: Option<PathBuf>,
        op_node_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            deploy_config_file: Some(
                deploy_config_file.unwrap_or(L2Genesis::get_deploy_config_file_unsafe()),
            ),
            deployment_dir: Some(deployment_dir.unwrap_or(L2Genesis::get_deployment_dir_unsafe())),
            l2_genesis_file: Some(
                l2_genesis_file.unwrap_or(L2Genesis::get_genesis_l2_file_unsafe()),
            ),
            genesis_rollup_file: Some(
                genesis_rollup_file.unwrap_or(L2Genesis::get_genesis_rollup_file_unsafe()),
            ),
            op_node_dir: Some(op_node_dir.unwrap_or(L2Genesis::get_op_node_dir_unsafe())),
        }
    }

    /// Returns a [PathBuf] for the Deploy Config Directory.
    ///
    /// # Panics
    ///
    /// Panics if the [project_root::get_project_root] function call fails to return a valid
    /// project root [PathBuf].
    pub fn get_deploy_config_file_unsafe() -> PathBuf {
        let proj_root = project_root::get_project_root().expect("Failed to get project root");
        let op_up_dir = proj_root.as_path();
        let contracts_bedrock_dir = op_up_dir
            .join("optimism")
            .join("packages/contracts-bedrock");
        let deploy_config_dir = contracts_bedrock_dir.join("deploy-config");
        deploy_config_dir.join("devnetL1.json")
    }

    /// Returns a [PathBuf] for the deployment dir.
    ///
    /// # Panics
    ///
    /// Panics if the [project_root::get_project_root] function call fails to return a valid
    /// project root [PathBuf].
    pub fn get_deployment_dir_unsafe() -> PathBuf {
        let proj_root = project_root::get_project_root().expect("Failed to get project root");
        let op_up_dir = proj_root.as_path();
        let contracts_bedrock_dir = op_up_dir
            .join("optimism")
            .join("packages/contracts-bedrock");
        contracts_bedrock_dir.join("deployments/devnetL1")
    }

    /// Returns a [PathBuf] for the genesis l2 file.
    ///
    /// # Panics
    ///
    /// Panics if the [project_root::get_project_root] function call fails to return a valid
    /// project root [PathBuf].
    pub fn get_genesis_l2_file_unsafe() -> PathBuf {
        let proj_root = project_root::get_project_root().expect("Failed to get project root");
        let op_up_dir = proj_root.as_path();
        op_up_dir.join(".devnet").join("genesis-l2.json")
    }

    /// Returns a [PathBuf] for the genesis rollup file.
    ///
    /// # Panics
    ///
    /// Panics if the [project_root::get_project_root] function call fails to return a valid
    /// project root [PathBuf].
    pub fn get_genesis_rollup_file_unsafe() -> PathBuf {
        let proj_root = project_root::get_project_root().expect("Failed to get project root");
        let op_up_dir = proj_root.as_path();
        op_up_dir.join(".devnet").join("rollup.json")
    }

    /// Returns a [PathBuf] for the op node dir.
    ///
    /// # Panics
    ///
    /// Panics if the [project_root::get_project_root] function call fails to return a valid
    /// project root [PathBuf].
    pub fn get_op_node_dir_unsafe() -> PathBuf {
        let proj_root = project_root::get_project_root().expect("Failed to get project root");
        let op_up_dir = proj_root.as_path();
        op_up_dir.join("optimism").join("op-node")
    }
}
