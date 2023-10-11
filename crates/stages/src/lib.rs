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
pub const OP_NODE_URL: &str = "http://localhost:7545";

/// Rollup node port.
pub const OP_NODE_PORT: u16 = 7545;

/// Testing deployer private key.
pub const DEPLOYER_PRIVATE_KEY: &str =
    "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

/// Stages
///
/// This module contains the code for the stages of the stack.
#[derive(Debug, Clone)]
pub struct Stages<'a> {
    /// The stack config.
    pub config: Config<'a>,
}

impl Stages<'_> {
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
        let allocs_file = devnet_dir.join("allocs-l1.json");
        let addresses_json_file = devnet_dir.join("addresses.json");
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
        if !devnet_dir.exists() {
            std::fs::create_dir_all(&devnet_dir)?;
        }

        // Step 1.
        // Create prestate and allocs

        // TODO: is this condition correct?
        if !genesis_l2_file.exists() {
            tracing::info!(target: "opup", "Making prestate and allocs...");
            let bin_dir = op_monorepo_dir.join("op-program/bin");
            if std::fs::metadata(bin_dir).is_err() {
                let make_command = Command::new("make")
                    .args(["cannon-prestate"])
                    .current_dir(&op_monorepo_dir)
                    .output()?;
                check_command(make_command, "Failed to do cannon prestate")?;
            }
            // TODO: check is allocs here actually do what we want
            let allocs = Command::new("make")
                .args(["devnet-allocs"])
                .current_dir(&op_monorepo_dir)
                .output()?;
            check_command(allocs, "Failed to do allocs")?;
            let copy_addr = Command::new("cp")
                .args([".devnet/addresses.json", "../.devnet/"])
                .current_dir(&op_monorepo_dir)
                .output()?;
            check_command(copy_addr, "Failed to do copy of addresses.json")?;
            let copy_allocs = Command::new("cp")
                .args([".devnet/allocs-l1.json", "../.devnet/"])
                .current_dir(&op_monorepo_dir)
                .output()?;
            check_command(copy_allocs, "Failed to do copy of allocs.json")?;
        }

        // Step 2.
        // Generate deploy config

        tracing::info!(target: "opup", "Generating deploy config...");
        let mut deploy_config = json::read_json(&deploy_config_file)?;
        let hex_timestamp = format!("{:#x}", curr_timestamp);
        json::set_json_property(&mut deploy_config, "l1GenesisBlockTimestamp", hex_timestamp);
        json::set_json_property(&mut deploy_config, "l1StartingBlockTag", "earliest");
        json::write_json(&deploy_config_file, &deploy_config)?;

        // Step 3.
        // Create L1 genesis

        if !genesis_l1_file.exists() {
            tracing::info!(target: "opup", "Creating L1 genesis...");
            let genesis_template = genesis::genesis_template_string(curr_timestamp)
                .ok_or_else(|| eyre::eyre!("Could not create genesis template"))?;
            std::fs::write(genesis_l1_file.clone(), genesis_template)?;
            let l1_genesis = Command::new("go")
                .args(["run", "cmd/main.go", "genesis", "l1"])
                .args(["--deploy-config", deploy_config_file.to_str().unwrap()])
                .args(["--l1-allocs", allocs_file.to_str().unwrap()])
                .args(["--l1-deployments", addresses_json_file.to_str().unwrap()])
                .args(["--outfile.l1", genesis_l1_file.to_str().unwrap()])
                .current_dir(&op_node_dir)
                .output()?;
            check_command(l1_genesis, "Failed to create L1 genesis")?;
        } else {
            tracing::info!(target: "opup", "L1 genesis already found.");
        }

        // Step 4.
        // Start L1 execution client

        tracing::info!(target: "opup", "Starting L1 execution client...");
        let start_l1 = Command::new("docker-compose")
            .args(["up", "-d", "l1"])
            .env("PWD", docker_dir.to_str().unwrap())
            .env("L1_CLIENT_CHOICE", self.config.l1_client.to_string())
            .current_dir(&docker_dir)
            .output()?;

        check_command(start_l1, "Failed to start L1 execution client")?;
        net::wait_up(L1_PORT, 10, 1)?;
        // TODO: is this sleep necessary?
        std::thread::sleep(std::time::Duration::from_secs(10));

        // Step 5.
        // Bind the addresses (contracts already deployed by "allocs" step)

        let addresses = json::read_json(&addresses_json_file)?;

        // Step 6.
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

        // Step 7.
        // Start L2 execution client

        tracing::info!(target: "opup", "Starting L2 execution client...");
        let start_l2 = Command::new("docker-compose")
            .args(["up", "-d", "l2"])
            .env("PWD", docker_dir.to_str().unwrap())
            .env("L2_CLIENT_CHOICE", self.config.l2_client.to_string())
            .current_dir(&docker_dir)
            .output()?;
        check_command(start_l2, "Failed to start L2 execution client")?;
        net::wait_up(L2_PORT, 10, 1)?;

        // Step 8.
        // Start other services

        tracing::info!(target: "opup", "Bringing up everything else...");
        let rollup_config = json::read_json(&genesis_rollup_file)?;
        let start_rollup = Command::new("docker-compose")
            .args(["up", "-d", "node", "proposer", "batcher"])
            .env("PWD", docker_dir.to_str().unwrap())
            .env("L2OO_ADDRESS", addresses["L2OutputOracleProxy"].to_string())
            .env(
                "SEQUENCER_BATCH_INBOX_ADDRESS",
                rollup_config["batch_inbox_address"].to_string(),
            )
            .current_dir(&docker_dir)
            .output()?;
        check_command(start_rollup, "Failed to start rollup client")?;
        net::wait_up(OP_NODE_PORT, 30, 1)?;

        // Done!

        tracing::info!(target: "opup", "--------------------------");
        tracing::info!(target: "opup", "Devnet built successfully!");
        tracing::info!(target: "opup", "L1 endpoint: {}", L1_URL);
        tracing::info!(target: "opup", "L2 endpoint: {}", L2_URL);
        tracing::info!(target: "opup", "op-node endpoint: {}", OP_NODE_URL);
        tracing::info!(target: "opup", "--------------------------\n");

        Ok(())
    }
}

impl<'a> From<Config<'a>> for Stages<'a> {
    fn from(config: Config<'a>) -> Self {
        Self { config }
    }
}
