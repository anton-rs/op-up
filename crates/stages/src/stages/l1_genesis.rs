use std::path::PathBuf;
use std::process::Command;

use eyre::Result;

/// L1 Genesis Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct L1Genesis {
    /// The path to the deployment config file.
    pub deploy_config_file: Option<PathBuf>,
    /// The l1 genesis file path.
    pub l1_genesis_file: Option<PathBuf>,
    /// The allocs file.
    pub allocs_file: Option<PathBuf>,
    /// The l1 deployments file.
    pub addresses_json: Option<PathBuf>,
    /// The op node directory.
    pub op_node_dir: Option<PathBuf>,
    /// The l1 genesis timestamp.
    pub genesis_timestamp: u64,
}

impl crate::Stage for L1Genesis {
    /// Executes the [L1Genesis] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing l1 genesis stage");

        let deploy_config_file = self
            .deploy_config_file
            .as_ref()
            .and_then(|p| p.to_str())
            .ok_or(eyre::eyre!("missing deploy config file"))?;

        let allocs_file = self
            .allocs_file
            .as_ref()
            .and_then(|p| p.to_str())
            .ok_or(eyre::eyre!("missing allocs file"))?;

        let l1_genesis_file = self
            .l1_genesis_file
            .as_ref()
            .ok_or(eyre::eyre!("missing l1 genesis file"))?;

        let addresses_json = self
            .addresses_json
            .as_ref()
            .and_then(|p| p.to_str())
            .ok_or(eyre::eyre!("missing l1 deployments file"))?;

        let op_node_dir = self
            .op_node_dir
            .as_ref()
            .and_then(|p| p.to_str())
            .ok_or(eyre::eyre!("missing op node directory"))?;

        if l1_genesis_file.exists() {
            tracing::info!(target: "stages", "L1 genesis already found.");
            return Ok(());
        }

        let l1_genesis_file_str = l1_genesis_file
            .to_str()
            .unwrap_or("failed to stringify l1 genesis file");

        tracing::info!(target: "stages", "Creating L1 genesis...");
        let genesis_template =
            op_primitives::genesis::genesis_template_string(self.genesis_timestamp)
                .ok_or_else(|| eyre::eyre!("Could not create genesis template"))?;
        std::fs::write(l1_genesis_file, genesis_template)?;
        let l1_genesis = Command::new("go")
            .args(["run", "cmd/main.go", "genesis", "l1"])
            .args(["--deploy-config", deploy_config_file])
            .args(["--l1-allocs", allocs_file])
            .args(["--l1-deployments", addresses_json])
            .args(["--outfile.l1", l1_genesis_file_str])
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
    pub fn new(
        l1_genesis_file: Option<PathBuf>,
        deploy_config_file: Option<PathBuf>,
        allocs_file: Option<PathBuf>,
        addresses_json: Option<PathBuf>,
        op_node_dir: Option<PathBuf>,
        genesis_timestamp: u64,
    ) -> Self {
        Self {
            l1_genesis_file: Some(
                l1_genesis_file.unwrap_or(L1Genesis::get_l1_genesis_file_unsafe()),
            ),
            deploy_config_file: Some(
                deploy_config_file.unwrap_or(L1Genesis::get_deploy_config_file_unsafe()),
            ),
            allocs_file: Some(allocs_file.unwrap_or(L1Genesis::get_allocs_file_unsafe())),
            addresses_json: Some(
                addresses_json.unwrap_or(L1Genesis::get_addresses_json_file_unsafe()),
            ),
            op_node_dir: Some(op_node_dir.unwrap_or(L1Genesis::get_op_node_dir_unsafe())),
            genesis_timestamp,
        }
    }

    /// Returns a [PathBuf] for the l1 genesis file.
    ///
    /// # Panics
    ///
    /// Panics if the [project_root::get_project_root] function call fails to return a valid
    /// project root [PathBuf].
    pub fn get_l1_genesis_file_unsafe() -> PathBuf {
        let proj_root = project_root::get_project_root().expect("Failed to get project root");
        let op_up_dir = proj_root.as_path();
        op_up_dir.join(".devnet").join("genesis-l1.json")
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

    /// Returns a [PathBuf] for the allocs file.
    ///
    /// # Panics
    ///
    /// Panics if the [project_root::get_project_root] function call fails to return a valid
    /// project root [PathBuf].
    pub fn get_allocs_file_unsafe() -> PathBuf {
        let proj_root = project_root::get_project_root().expect("Failed to get project root");
        let op_up_dir = proj_root.as_path();
        op_up_dir.join(".devnet").join("allocs-l1.json")
    }

    /// Returns a [PathBuf] for the addresses json file.
    ///
    /// # Panics
    ///
    /// Panics if the [project_root::get_project_root] function call fails to return a valid
    /// project root [PathBuf].
    pub fn get_addresses_json_file_unsafe() -> PathBuf {
        let proj_root = project_root::get_project_root().expect("Failed to get project root");
        let op_up_dir = proj_root.as_path();
        op_up_dir.join(".devnet").join("addresses.json")
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
