use bollard::Docker;
use eyre::Result;
use std::path::Path;
use std::process::Command;

use op_config::Config;
use op_stack::genesis;

use crate::{
    addresses, constants,
    etc::{
        clock,
        commands::{self, check_command},
        git, json, net, runner,
    },
};

/// Spin up the stack.
pub fn run() -> Result<()> {
    runner::run_until_ctrl_c(async {
        let docker = Docker::connect_with_local_defaults()?;
        let version = docker.version().await?;
        tracing::info!(target: "opup", "docker version: {:?}", version);

        crate::banners::banner()?;

        temp()?;

        Ok(())
    })
}

/// Temporary function to port the old cli.
pub fn temp() -> Result<()> {
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
    let op_rs_monorepo_dir = op_up_dir.join("optimism-rs");

    // Files referenced
    let genesis_l1_file = devnet_dir.join("genesis-l1.json");
    let genesis_l2_file = devnet_dir.join("genesis-l2.json");
    let genesis_rollup_file = devnet_dir.join("rollup.json");
    let addresses_json_file = devnet_dir.join("addresses.json");
    let addresses_sdk_json_file = devnet_dir.join("addresses-sdk.json");
    let deploy_config_file = deploy_config_dir.join("devnetL1.json");

    // Create a new op-stack config object from user choices
    // (or load an existing one from the .stack file if it exists)
    tracing::info!(target: "opup", "Loading op-stack config...");
    let current_dir = std::env::current_dir()?;
    let stack = Config::load_with_root(current_dir);

    // Check if the optimism and optimism-rs paths exist in the project root dir.
    // If not, clone them from Github
    if !Path::new(&op_monorepo_dir).exists() {
        tracing::info!(target: "opup", "Cloning the optimism monorepo from github (this may take a while)...");
        git::git_clone(op_up_dir, constants::OP_MONOREPO_URL)?;
    }
    // There is no more optimism-rs monorepo :{
    if !Path::new(&op_rs_monorepo_dir).exists() {
        tracing::info!(target: "opup", "Cloning the optimism-rs monorepo from github (this may take a while)...");
        // git::git_clone(op_up_dir, constants::OP_RS_MONOREPO_URL)?;
    }

    // ----------------------------------------
    // Build the devnet

    // Step 0.
    // Setup

    tracing::info!(target: "opup", "Building devnet...");
    std::fs::create_dir_all(devnet_dir)?;
    let curr_timestamp = clock::current_timestamp();
    let genesis_template = genesis::genesis_template_string(curr_timestamp)
        .ok_or_else(|| eyre::eyre!("Could not create genesis template"))?;

    // Step 1.
    // Create L1 genesis
    if !genesis_l1_file.exists() {
        tracing::info!(target: "opup", "Creating L1 genesis...");
        std::fs::write(genesis_l1_file, genesis_template)?;
    } else {
        tracing::info!(target: "opup", "L1 genesis already found.");
    }

    // Step 2.
    // Start L1 execution client

    tracing::info!(target: "opup", "Starting L1 execution client...");
    let start_l1 = Command::new("docker-compose")
        .args(["up", "-d", "--no-deps", "--build", "l1"])
        .env("PWD", docker_dir.to_str().unwrap())
        .env("L1_CLIENT_CHOICE", stack.l1_client.to_string())
        .current_dir(&docker_dir)
        .output()?;

    commands::check_command(start_l1, "Failed to start L1 execution client")?;
    net::wait_up(constants::L1_PORT, 10, 1)?;

    if !genesis_l2_file.exists() {
        tracing::info!(target: "opup", "Creating L2 and rollup genesis...");
        let l2_genesis = Command::new("make")
            .args(["devnet-allocs"])
            .current_dir(&op_monorepo_dir)
            .output()?;
        check_command(l2_genesis, "Failed to create L2 genesis")?;
    }

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
        println!("Deploying contracts...");
        let install_deps = Command::new("yarn")
            .args(["install"])
            .current_dir(&contracts_bedrock_dir)
            .output()?;
        check_command(install_deps, "Failed to install dependencies")?;

        let deploy_contracts = Command::new("yarn")
            .args(["hardhat", "--network", "devnetL1", "deploy", "--tags", "l1"])
            .env("CHAIN_ID", "900")
            .env("L1_RPC", constants::L1_URL)
            .env("PRIVATE_KEY_DEPLOYER", constants::DEPLOYER_PRIVATE_KEY)
            .current_dir(&contracts_bedrock_dir)
            .output()?;

        check_command(deploy_contracts, "Failed to deploy contracts")?;

        // Write the addresses to json
        let (addresses, sdk_addresses) = addresses::set_addresses(&deployment_dir)?;
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
            .args(["--l1-rpc", constants::L1_URL])
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
        .env("L2_CLIENT_CHOICE", stack.l2_client.to_string())
        .current_dir(&docker_dir)
        .output()?;
    check_command(start_l2, "Failed to start L2 execution client")?;
    net::wait_up(constants::L2_PORT, 10, 1)?;

    // Step 7.
    // Start rollup client

    println!("Starting rollup client...");
    let start_rollup = Command::new("docker-compose")
        .args(["up", "-d", "--no-deps", "--build", "rollup-client"])
        .env("PWD", docker_dir.to_str().unwrap())
        .env("ROLLUP_CLIENT_CHOICE", stack.rollup_client.to_string())
        .current_dir(&docker_dir)
        .output()?;
    check_command(start_rollup, "Failed to start rollup client")?;
    net::wait_up(constants::ROLLUP_PORT, 30, 1)?;

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
        .env("CHALLENGER_AGENT_CHOICE", stack.challenger.to_string())
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
    println!("L1 endpoint: {}", constants::L1_URL);
    println!("L2 endpoint: {}", constants::L2_URL);
    println!("Rollup node endpoint: {}", constants::ROLLUP_URL);
    println!("--------------------------\n");

    Ok(())
}
