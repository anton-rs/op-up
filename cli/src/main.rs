use std::{fs, path::Path, process::Command};

use eyre::{eyre, Result};
use inquire::Confirm;
use serde_json::Map;

use crate::stack::OpStackConfig;

mod constants;
mod genesis;
mod stack;
mod utils;

fn main() -> Result<()> {
    println!(
        r#"                        
                                         
        ___   _____            __  __  _____   
       / __`\/\ '__`\  _______/\ \/\ \/\ '__`\ 
      /\ \L\ \ \ \L\ \/\______\ \ \_\ \ \ \L\ \
      \ \____/\ \ ,__/\/______/\ \____/\ \ ,__/
       \/___/  \ \ \/           \/___/  \ \ \/ 
                \ \_\                    \ \_\ 
                 \/_/                     \/_/ 
      
      "#
    );

    let cwd = std::env::current_dir()?;
    let op_up_dir = cwd.parent().ok_or(eyre!("Failed to get project root"))?;

    // Directories referenced
    let docker_dir = op_up_dir.join("docker");
    let devnet_dir = op_up_dir.join(".devnet");
    let scripts_dir = op_up_dir.join("scripts");
    let op_monorepo_dir = op_up_dir.join("optimism");
    let op_node_dir = op_monorepo_dir.join("op-node");
    let ops_bedrock_dir = op_monorepo_dir.join("ops-bedrock");
    let contracts_bedrock_dir = op_monorepo_dir.join("packages/contracts-bedrock");
    let deploy_config_dir = contracts_bedrock_dir.join("deploy-config");
    let deployment_dir = contracts_bedrock_dir.join("deployments/devnetL1");
    let op_rs_monorepo_dir = op_up_dir.join("optimism-rs");

    // Files referenced
    let stack_file = op_up_dir.join(".stack");
    let genesis_l1_file = devnet_dir.join("genesis_l1.json");
    let genesis_l2_file = devnet_dir.join("genesis_l2.json");
    let genesis_rollup_file = devnet_dir.join("rollup.json");
    let addresses_json_file = devnet_dir.join("addresses.json");
    let addresses_sdk_json_file = devnet_dir.join("addresses_sdk.json");
    let deploy_config_file = deploy_config_dir.join("devnetL1.json");
    let devnet_up_script = scripts_dir.join("main.py");

    // ----------------------------------------
    // Create a new op-stack config object from user choices
    // (or load an existing one from the .stack file if it exists)

    let stack = if stack_file.exists() {
        let existing_stack = stack::read_from_file(&stack_file)?;
        println!("Looks like you've already got an existing op-stack loaded!");

        let use_existing = Confirm::new("Do you want to use the existing stack?")
            .with_default(true)
            .with_help_message(existing_stack.to_string().as_str())
            .prompt()?;

        if use_existing {
            println!("\nGreat! We'll use the existing stack.");
            existing_stack
        } else {
            fs::remove_file(&stack_file)?;
            println!("\nOk, we'll start from scratch then.");
            OpStackConfig::from_user_choices()?
        }
    } else {
        println!("\nWelcome to the interactive op-stack devnet builder!");
        println!("Please select your desired op-stack components:\n");
        OpStackConfig::from_user_choices()?
    };

    // Remember the selected stack for next time
    stack::write_to_file(&stack_file, &stack)?;

    // Check if the optimism and optimism-rs paths exist in the project root dir.
    // If not, clone them from Github
    if !Path::new(&op_monorepo_dir).exists() {
        println!("Cloning the optimism monorepo from github (this may take a while)...");
        utils::git_clone(op_up_dir, constants::OP_MONOREPO_URL)?;
    }
    if !Path::new(&op_rs_monorepo_dir).exists() {
        println!("Cloning the optimism-rs monorepo from github (this may take a while)...");
        utils::git_clone(op_up_dir, constants::OP_RS_MONOREPO_URL)?;
    }

    // ----------------------------------------
    // Build the devnet

    println!("Building devnet...");

    // // Run the main orchestration script
    // let devnet_up = Command::new(devnet_up_script)
    //     .env("L1_CLIENT_CHOICE", stack.l1_client.to_string())
    //     .env("L2_CLIENT_CHOICE", stack.l2_client.to_string())
    //     .env("ROLLUP_CLIENT_CHOICE", stack.rollup_client.to_string())
    //     .env("CHALLENGER_AGENT_CHOICE", stack.challenger.to_string())
    //     .current_dir(&docker_dir)
    //     .output()?;

    // utils::check_command(devnet_up, "Failed to build devnet")?;

    // Step 0.
    // Setup

    fs::create_dir_all(devnet_dir)?;
    let curr_timestamp = utils::current_timestamp();
    let genesis_template = genesis::genesis_template(curr_timestamp);

    // Step 1.
    // Create L1 genesis

    if !genesis_l1_file.exists() {
        println!("Creating L1 genesis...");
        fs::write(genesis_l1_file, genesis_template)?;
    } else {
        println!("L1 genesis already found.");
    }

    // Step 2.
    // Start L1 execution client

    println!("Starting L1 execution client...");
    let start_l1 = Command::new("docker-compose")
        .args(["up", "-d", "--no-deps", "--build", "l1"])
        .env("L1_CLIENT_CHOICE", stack.l1_client.to_string())
        .current_dir(&docker_dir)
        .output()?;

    utils::check_command(start_l1, "Failed to start L1 execution client")?;
    utils::wait_up(constants::L1_PORT, 10, 1)?;

    // Step 3.
    // Generate network configs

    println!("Generating network configs...");
    let mut deploy_config = utils::read_json(&deploy_config_file)?;
    utils::set_json_property(
        &mut deploy_config,
        "l1GenesisBlockTimestamp",
        curr_timestamp,
    );
    utils::set_json_property(&mut deploy_config, "l1StartingBlockTag", "earliest");
    utils::write_json(&deploy_config_file, &deploy_config)?;

    // Step 4.
    // Deploy contracts

    if !addresses_json_file.exists() {
        println!("Deploying contracts...");
        let deploy_contracts = Command::new("yarn")
            .args(["hardhat", "--network", "devnetL1", "deploy", "--tags", "l1"])
            .env("CHAIN_ID", "900")
            .env("L1_RPC", constants::L1_URL)
            .env("PRIVATE_KEY_DEPLOYER", constants::DEPLOYER_PRIVATE_KEY)
            .current_dir(&contracts_bedrock_dir)
            .output()?;

        utils::check_command(deploy_contracts, "Failed to deploy contracts")?;
    }

    let mut addresses = Map::new();
    let entries = fs::read_dir(deployment_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().unwrap_or_default() == "json" {
            let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
            let data = utils::read_json(&path)?;

            if let Some(address) = data["address"].as_str() {
                addresses.insert(file_name, address.to_owned().into());
            }
        }
    }

    Ok(())
}
