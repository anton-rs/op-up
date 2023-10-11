use std::path::PathBuf;

use eyre::Result;

/// Contract Deployment Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Contracts;

impl crate::Stage for Contracts {
    /// Executes the [Contracts] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing contract deployment stage");

        // contract deployment is already done in allocs stage
        // if we want to allow for other contract implementations as part of the stack
        // they should be added here.

        // let contracts_bedrock_dir = proj_root
        //     .as_path()
        //     .join("optimism")
        //     .join("packages/contracts-bedrock");
        // let addresses_json_file = proj_root.as_path().join(".devnet").join("addresses.json");
        // let addresses_sdk_json_file = proj_root
        //     .as_path()
        //     .join(".devnet")
        //     .join("addresses-sdk.json");
        // if !addresses_json_file.exists() {
        //     tracing::info!(target: "stages", "Deploying contracts...");
        //     let install_deps = Command::new("yarn")
        //         .args(["install"])
        //         .current_dir(&contracts_bedrock_dir)
        //         .output()?;
        //     if !install_deps.status.success() {
        //         eyre::bail!(
        //             "failed to install dependencies: {}",
        //             String::from_utf8_lossy(&install_deps.stderr)
        //         )
        //     }
        //
        //     let deployer = self
        //         .config
        //         .deployer
        //         .as_ref()
        //         .ok_or_else(|| eyre::eyre!("missing contracts deployer"))?;
        //     let deploy_contracts = Command::new("yarn")
        //         .args(["hardhat", "--network", "devnetL1", "deploy", "--tags", "l1"])
        //         .env("CHAIN_ID", "900")
        //         .env("L1_RPC", op_config::L1_URL)
        //         .env("PRIVATE_KEY_DEPLOYER", deployer)
        //         .current_dir(&contracts_bedrock_dir)
        //         .output()?;
        //     if !deploy_contracts.status.success() {
        //         eyre::bail!(
        //             "failed to deploy contracts: {}",
        //             String::from_utf8_lossy(&deploy_contracts.stderr)
        //         )
        //     }
        //
        //     // Write the addresses to json
        //     let deployment_dir =
        //         op_monorepo_dir.join("packages/contracts-bedrock/deployments/devnetL1");
        //     let (addresses, sdk_addresses) = AddressManager::set_addresses(&deployment_dir)?;
        //     crate::json::write_json(&addresses_json_file, &addresses)?;
        //     crate::json::write_json(&addresses_sdk_json_file, &sdk_addresses)?;
        // }

        Ok(())
    }
}

impl Contracts {
    /// Creates a new stage.
    pub fn new() -> Self {
        Self::default()
    }
}
