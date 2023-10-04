#![doc = include_str!("../README.md")]
#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    rustdoc::all
)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use std::path::Path;
use std::process::Command;

use op_config::Config;
use op_contracts::AddressManager;
use op_primitives::genesis;

mod commands;
use commands::check_command;

mod git;
mod json;
mod net;

/// Optimism monorepo git url.
pub const OP_MONOREPO_URL: &str = "git@github.com:ethereum-optimism/optimism.git";

/// L1 node url.
pub const L1_URL: &str = "http://localhost:8545";

/// L1 node port.
pub const L1_PORT: u16 = 8545;

/// L2 node url.
pub const L2_URL: &str = "http://localhost:9545";

/// L2 node port.
pub const L2_PORT: u16 = 9545;

/// Rollup node url.
pub const ROLLUP_URL: &str = "http://localhost:7545";

/// Rollup node port.
pub const ROLLUP_PORT: u16 = 7545;

/// Testing deployer private key.
pub const DEPLOYER_PRIVATE_KEY: &str =
    "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

/// Stages
///
/// This module contains the code for the stages of the stack.
#[derive(Debug, Clone)]
pub struct Stages {
    /// The stack config.
    pub config: Config,
}

impl Stages {
    /// Execute the stages of the stack.
    pub async fn execute(&self) -> eyre::Result<()> {
        tracing::debug!(target: "opup", "bootstrapping op stack");

        let proj_root = project_root::get_project_root()?;
        let op_up_dir = proj_root.as_path();

        // Directories referenced
        let docker_dir = op_up_dir.join("docker");
        let devnet_dir = op_up_dir.join(".devnet");
        let op_monorepo_dir = op_up_dir.join("optimism");
        let op_node_dir = op_monorepo_dir.join("op-node");
        let contracts_bedrock_dir = op_monorepo_dir.join("packages/contracts-bedrock");
        let deploy_config_dir = contracts_bedrock_dir.join("deploy-config");
        let deployment_dir = contracts_bedrock_dir.join("deployments/devnetL1");

        // Files referenced
        let genesis_l1_file = devnet_dir.join("genesis-l1.json");
        let genesis_l2_file = devnet_dir.join("genesis-l2.json");
        let genesis_rollup_file = devnet_dir.join("rollup.json");
        let addresses_json_file = devnet_dir.join("addresses.json");
        let addresses_sdk_json_file = devnet_dir.join("addresses-sdk.json");
        let deploy_config_file = deploy_config_dir.join("devnetL1.json");

        // Check if the optimism and optimism-rs paths exist in the project root dir.
        // If not, clone them from Github
        if !Path::new(&op_monorepo_dir).exists() {
            tracing::info!(target: "opup", "Cloning the optimism monorepo from github (this may take a while)...");
            git::git_clone(op_up_dir, OP_MONOREPO_URL)?;
        }

        // ----------------------------------------
        // Build the devnet

        // Step 0.
        // Setup

        tracing::info!(target: "opup", "Building devnet...");
        self.config.create_artifacts_dir()?;
        let curr_timestamp = genesis::current_timestamp();

        // Step 1.
        // Create L1 genesis
        if !genesis_l1_file.exists() {
            tracing::info!(target: "opup", "Creating L1 genesis...");
            let genesis_template = genesis::genesis_template_string(curr_timestamp)
                .ok_or_else(|| eyre::eyre!("Could not create genesis template"))?;
            if let Some(parent_dir) = genesis_l1_file.parent() {
                std::fs::create_dir_all(parent_dir)?;
            }
            std::fs::write(genesis_l1_file, genesis_template)?;
        } else {
            tracing::info!(target: "opup", "L1 genesis already found.");
        }

        if !genesis_l2_file.exists() {
            tracing::info!(target: "opup", "Do some stuff to get the L2 genesis...");

            let bin_dir = op_monorepo_dir.join("op-program/bin");
            if !std::fs::metadata(bin_dir).is_ok() {
                let make_command = Command::new("make")
                    .args(["cannon-prestate"])
                    .current_dir(&op_monorepo_dir)
                    .output()?;
                check_command(make_command, "Failed to do cannon prestate")?;
            }
            // TODO: check is allocs here actually do what we want
            // let allocs = Command::new("make")
            //     .args(["devnet-allocs"])
            //     .current_dir(&op_monorepo_dir)
            //     .output()?;
            // check_command(allocs, "Failed to do allocs")?;
        }

        // Step 2.
        // Start L1 execution client

        tracing::info!(target: "opup", "Starting L1 execution client...");
        let start_l1 = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "l1"])
            .env("PWD", docker_dir.to_str().unwrap())
            .env("L1_CLIENT_CHOICE", self.config.l1_client.to_string())
            .current_dir(&docker_dir)
            .output()?;

        check_command(start_l1, "Failed to start L1 execution client")?;
        net::wait_up(L1_PORT, 10, 1)?;

        // Step 3.
        // Generate network configs
        tracing::info!(target: "opup", "Generating network configs...");
        let mut deploy_config = json::read_json(&deploy_config_file)?;
        json::set_json_property(
            &mut deploy_config,
            "l1GenesisBlockTimestamp",
            curr_timestamp,
        );
        json::set_json_property(&mut deploy_config, "l1StartingBlockTag", "earliest");
        json::write_json(&deploy_config_file, &deploy_config)?;

        // Step 4.
        // Deploy contracts
        let addresses = if !addresses_json_file.exists() {
            tracing::info!(target: "opup", "Deploying contracts...");
            let install_deps = Command::new("pnpm")
                .args(["install"])
                .current_dir(&contracts_bedrock_dir)
                .output()?;
            check_command(install_deps, "Failed to install dependencies")?;

            // let deploy_contracts = Command::new("yarn")
            //     .args(["hardhat", "--network", "devnetL1", "deploy", "--tags", "l1"])
            //     .env("CHAIN_ID", "900")
            //     .env("L1_RPC", L1_URL)
            //     .env("PRIVATE_KEY_DEPLOYER", DEPLOYER_PRIVATE_KEY)
            //     .current_dir(&contracts_bedrock_dir)
            //     .output()?;
            let deploy_contracts = {
                let send_eth = Command::new("cast")
                    .args(["send"])
                    .args(["--from", "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"])
                    .args(["--private-key", "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"])
                    .args(["--rpc-url", L1_URL])
                    .args(["--unlocked", "--value", "1ether", "0x3fAB184622Dc19b6109349B94811493BF2a45362"])
                    .current_dir(&contracts_bedrock_dir)
                    .output()?;
                check_command(send_eth, "Failed to send eth to create2")?;

                let deploy_1 = Command::new("cast")
                    .args(["publish"])
                    .args(["--rpc-url", L1_URL])
                    .args(["--private-key", "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"])
                    .args(["0xf8a58085174876e800830186a08080b853604580600e600039806000f350fe7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe03601600081602082378035828234f58015156039578182fd5b8082525050506014600cf31ba02222222222222222222222222222222222222222222222222222222222222222a02222222222222222222222222222222222222222222222222222222222222222"])
                    .current_dir(&contracts_bedrock_dir)
                    .output()?;
                check_command(deploy_1, "Failed to deploy the create2 deployer")?;

                let deploy_2 = Command::new("forge")
                    .args(["script", "scripts/Deploy.s.sol:Deploy"])
                    .args(["--sender", "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"])
                    .args(["--private-key", "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"])
                    .args(["--rpc-url", L1_URL])
                    .args(["--broadcast", "--unlocked"])
                    .current_dir(&contracts_bedrock_dir)
                    .output()?;
                check_command(deploy_2, "Failed to deploy the create2 deployer (forge)")?;

                tracing::info!(target: "opup", "Syncing contracts.");
                Command::new("forge")
                    .args(["script", "scripts/Deploy.s.sol:Deploy"])
                    .args(["--sig", "sync()"])
                    .args(["--rpc-url", L1_URL])
                    .current_dir(&contracts_bedrock_dir)
                    .output()?
            };

            check_command(deploy_contracts, "Failed to deploy contracts")?;

            // Write the addresses to json
            let (addresses, sdk_addresses) = AddressManager::set_addresses(&deployment_dir)?;
            json::write_json(&addresses_json_file, &addresses)?;
            json::write_json(&addresses_sdk_json_file, &sdk_addresses)?;

            addresses
        } else {
            tracing::info!(target: "opup", "Contracts already deployed.");
            json::read_json(&addresses_json_file)?
        };

        // Step 5.
        // Create L2 genesis

        if !genesis_l2_file.exists() {
            tracing::info!(target: "opup", "Creating L2 and rollup genesis...");
            let l2_genesis = Command::new("go")
                .args(["run", "cmd/main.go", "genesis", "l2"])
                .args(["--l1-rpc", L1_URL])
                .args(["--deploy-config", deploy_config_file.to_str().unwrap()])
                .args(["--deployment-dir", deployment_dir.to_str().unwrap()])
                .args(["--outfile.l2", genesis_l2_file.to_str().unwrap()])
                .args(["--outfile.rollup", genesis_rollup_file.to_str().unwrap()])
                .current_dir(&op_node_dir)
                .output()?;
            check_command(l2_genesis, "Failed to create L2 genesis")?;
        } else {
            tracing::info!(target: "opup", "L2 genesis already found.");
        }

        // Step 6.
        // Start L2 execution client

        println!("Starting L2 execution client...");
        let start_l2 = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "l2"])
            .env("PWD", docker_dir.to_str().unwrap())
            .env("L2_CLIENT_CHOICE", self.config.l2_client.to_string())
            .current_dir(&docker_dir)
            .output()?;
        check_command(start_l2, "Failed to start L2 execution client")?;
        net::wait_up(L2_PORT, 10, 1)?;

        // Step 7.
        // Start rollup client

        println!("Starting rollup client...");
        let start_rollup = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "rollup-client"])
            .env("PWD", docker_dir.to_str().unwrap())
            .env(
                "ROLLUP_CLIENT_CHOICE",
                self.config.rollup_client.to_string(),
            )
            .current_dir(&docker_dir)
            .output()?;
        check_command(start_rollup, "Failed to start rollup client")?;
        net::wait_up(ROLLUP_PORT, 30, 1)?;

        // Step 8.
        // Start proposer

        println!("Starting proposer...");
        let start_proposer = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "proposer"])
            .env("PWD", docker_dir.to_str().unwrap())
            .env("L2OO_ADDRESS", addresses["L2OutputOracleProxy"].to_string())
            .current_dir(&docker_dir)
            .output()?;
        check_command(start_proposer, "Failed to start proposer")?;

        // Step 9.
        // Start batcher

        println!("Starting batcher...");
        let rollup_config = json::read_json(&genesis_rollup_file)?;
        let start_batcher = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "batcher"])
            .env("PWD", docker_dir.to_str().unwrap())
            .env("L2OO_ADDRESS", addresses["L2OutputOracleProxy"].to_string())
            .env(
                "SEQUENCER_BATCH_INBOX_ADDRESS",
                rollup_config["batch_inbox_address"].to_string(),
            )
            .current_dir(&docker_dir)
            .output()?;
        check_command(start_batcher, "Failed to start batcher")?;

        // Step 10.
        // Start challenger

        // TODO: Deploy the mock dispute game contract
        let dgf_address = "0x0000000000000000000000000000000000000000";

        println!("Starting challenger...");
        let start_challenger = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "challenger"])
            .env("PWD", docker_dir.to_str().unwrap())
            .env("L2OO_ADDRESS", addresses["L2OutputOracleProxy"].to_string())
            .env("DGF_ADDRESS", dgf_address)
            .env(
                "CHALLENGER_AGENT_CHOICE",
                self.config.challenger.to_string(),
            )
            .current_dir(&docker_dir)
            .output()?;
        check_command(start_challenger, "Failed to start challenger")?;

        // Step 11.
        // Start stateviz
        let start_stateviz = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "stateviz"])
            .env("PWD", docker_dir.to_str().unwrap())
            .env("L2OO_ADDRESS", addresses["L2OutputOracleProxy"].to_string())
            .current_dir(&docker_dir)
            .output()?;
        check_command(start_stateviz, "Failed to start stateviz")?;

        // Done!

        println!("\n--------------------------");
        println!("Devnet built successfully!");
        println!("L1 endpoint: {}", L1_URL);
        println!("L2 endpoint: {}", L2_URL);
        println!("Rollup node endpoint: {}", ROLLUP_URL);
        println!("--------------------------\n");

        Ok(())
    }
}

impl From<Config> for Stages {
    fn from(config: Config) -> Self {
        Self { config }
    }
}
