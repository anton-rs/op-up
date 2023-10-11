use std::path::Path;
use std::process::Command;

use eyre::Result;

use op_config::Config;
use op_contracts::AddressManager;
use op_primitives::genesis;

mod batcher;
mod challenger;
mod l2_genesis;
mod layer_two;
mod proposer;
mod rollup;
mod stateviz;

/// Stages
///
/// This module contains the code for the stages of the stack.
pub struct Stages<'a> {
    /// The stack config.
    pub config: Config<'a>,
    /// The inner stages.
    pub inner: Option<Vec<Box<dyn crate::Stage>>>,
}

impl Stages<'_> {
    /// Build the default docker-based stages.
    pub fn docker(&self) -> Vec<Box<dyn crate::Stage>> {
        vec![
            // Step 5: Create L2 Genesis (if it doesn't already exist)
            Box::new(l2_genesis::L2Genesis::new(None, None, None, None, None)),
            // Step 6: Start the L2 Execution Client
            Box::new(layer_two::LayerTwo::new(
                None,
                self.config.l2_client.to_string(),
            )),
            // Step 7: Start the Rollup Client
            Box::new(rollup::Rollup::new(
                None,
                self.config.rollup_client.to_string(),
            )),
            // Step 8: Start the Proposer
            Box::new(proposer::Proposer::new(None)),
            // Step 9: Start the Batcher
            Box::new(batcher::Batcher::new(None, None)),
            // Step 10: Start the Challenger Agent
            Box::new(challenger::Challenger::new(
                None,
                self.config.challenger.to_string(),
            )),
            // Step 11: Start State Vizualization Module
            Box::new(stateviz::Stateviz::new(None)),
        ]
    }

    /// Execute the stages of the stack.
    pub async fn execute(&self) -> eyre::Result<()> {
        tracing::debug!(target: "stages", "guts, glory, ramen");

        // todo: can this can be removed by having stages manage their own dependencies?
        // Check if the optimism  paths exist in the project root dir.
        let proj_root = project_root::get_project_root()?;
        let op_up_dir = proj_root.as_path();
        let op_monorepo_dir = op_up_dir.join("optimism");
        if !Path::new(&op_monorepo_dir).exists() {
            tracing::info!(target: "stages", "Cloning the optimism monorepo from github (this may take a while)...");
            crate::git::git_clone(op_up_dir, op_config::OP_MONOREPO_URL)?;
        }

        // Stage 0: Build the devnet
        self.config.create_artifacts_dir()?;
        let curr_timestamp = genesis::current_timestamp();
        let genesis_template = genesis::genesis_template_string(curr_timestamp)
            .ok_or_else(|| eyre::eyre!("Could not create genesis template"))?;

        // Stage 1: Create the L1 genesis
        let genesis_l1_file = proj_root.as_path().join(".devnet").join("genesis-l1.json");
        if !genesis_l1_file.exists() {
            tracing::info!(target: "stages", "Creating L1 genesis...");
            let genesis_l1_file = proj_root.as_path().join(".devnet").join("genesis-l1.json");
            std::fs::write(genesis_l1_file, genesis_template)?;
        } else {
            tracing::info!(target: "stages", "L1 genesis already found.");
        }

        // Stage 2: Start the L1 execution client
        tracing::info!(target: "stages", "Starting L1 execution client...");
        let docker_dir = proj_root.as_path().join("docker");
        let start_l1 = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "l1"])
            .env("PWD", docker_dir.to_str().unwrap())
            .env("L1_CLIENT_CHOICE", self.config.l1_client.to_string())
            .current_dir(&docker_dir)
            .output()?;

        if !start_l1.status.success() {
            eyre::bail!(
                "failed to start the l1 execution client: {}",
                String::from_utf8_lossy(&start_l1.stderr)
            );
        }

        crate::net::wait_up(op_config::L1_PORT, 10, 1)?;

        let genesis_l2_file = proj_root.as_path().join(".devnet").join("genesis-l2.json");
        if !genesis_l2_file.exists() {
            tracing::info!(target: "stages", "Creating L2 and rollup genesis...");
            let l2_genesis = Command::new("make")
                .args(["devnet-allocs"])
                .current_dir(&op_monorepo_dir)
                .output()?;
            if !l2_genesis.status.success() {
                eyre::bail!(
                    "failed to create L2 genesis: {}",
                    String::from_utf8_lossy(&l2_genesis.stderr)
                )
            }
        }

        // Step 3: Generate the network configs
        tracing::info!(target: "stages", "Generating network configs...");
        let deploy_config_file = proj_root
            .as_path()
            .join("optimism")
            .join("pacakges/contracts-bedrock")
            .join("deploy-config")
            .join("devnetL1.json");
        let mut deploy_config = crate::json::read_json(&deploy_config_file)?;
        crate::json::set_json_property(
            &mut deploy_config,
            "l1GenesisBlockTimestamp",
            curr_timestamp,
        );
        crate::json::set_json_property(&mut deploy_config, "l1StartingBlockTag", "earliest");
        crate::json::write_json(&deploy_config_file, &deploy_config)?;

        // Step 4: Deploy contracts
        let contracts_bedrock_dir = proj_root
            .as_path()
            .join("optimism")
            .join("packages/contracts-bedrock");
        let addresses_json_file = proj_root.as_path().join(".devnet").join("addresses.json");
        let addresses_sdk_json_file = proj_root
            .as_path()
            .join(".devnet")
            .join("addresses-sdk.json");
        if !addresses_json_file.exists() {
            tracing::info!(target: "stages", "Deploying contracts...");
            let install_deps = Command::new("yarn")
                .args(["install"])
                .current_dir(&contracts_bedrock_dir)
                .output()?;
            if !install_deps.status.success() {
                eyre::bail!(
                    "failed to install dependencies: {}",
                    String::from_utf8_lossy(&install_deps.stderr)
                )
            }

            let deployer = self
                .config
                .deployer
                .as_ref()
                .ok_or_else(|| eyre::eyre!("missing contracts deployer"))?;
            let deploy_contracts = Command::new("yarn")
                .args(["hardhat", "--network", "devnetL1", "deploy", "--tags", "l1"])
                .env("CHAIN_ID", "900")
                .env("L1_RPC", op_config::L1_URL)
                .env("PRIVATE_KEY_DEPLOYER", deployer)
                .current_dir(&contracts_bedrock_dir)
                .output()?;
            if !deploy_contracts.status.success() {
                eyre::bail!(
                    "failed to deploy contracts: {}",
                    String::from_utf8_lossy(&deploy_contracts.stderr)
                )
            }

            // Write the addresses to json
            let deployment_dir =
                op_monorepo_dir.join("packages/contracts-bedrock/deployments/devnetL1");
            let (addresses, sdk_addresses) = AddressManager::set_addresses(&deployment_dir)?;
            crate::json::write_json(&addresses_json_file, &addresses)?;
            crate::json::write_json(&addresses_sdk_json_file, &sdk_addresses)?;
        }

        // If there are no inner stages, construct the default docker build.
        let docker_stages = self.docker();
        let inner = self.inner.as_ref().unwrap_or_else(|| &docker_stages);

        // Execute the stages in order, synchronously.
        for stage in inner {
            stage.execute()?;
        }

        tracing::info!(target: "stages", "stages executed");
        Ok(())
    }

    /// Print the stack result to stdout.
    pub fn output(&self) -> Result<()> {
        // todo: get the actual stage output and print it here instead of using the defaults
        tracing::info!(target: "stages", "\n--------------------------");
        tracing::info!(target: "stages", "Devnet built successfully!");
        tracing::info!(target: "stages", "L1 endpoint: {}", op_config::L1_URL);
        tracing::info!(target: "stages", "L2 endpoint: {}", op_config::L2_URL);
        tracing::info!(target: "stages", "Rollup node endpoint: {}", op_config::ROLLUP_URL);
        tracing::info!(target: "stages", "--------------------------\n");
        Ok(())
    }
}

impl<'a> From<Config<'a>> for Stages<'a> {
    fn from(config: Config<'a>) -> Self {
        Self {
            config,
            inner: None,
        }
    }
}
