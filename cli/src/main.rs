use std::{env::current_dir, fmt::Display, fs::remove_file, path::Path, process::Command};

use eyre::{eyre, Result};
use inquire::Confirm;

use crate::{
    stack::{ChallengerAgent, L1Client, L2Client, RollupClient},
    utils::GitCloneMethod,
};

mod constants;
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

    let cwd = current_dir()?;
    let op_up_dir = cwd.parent().ok_or(eyre!("Failed to get project root"))?;

    // Directories referenced
    let ops_dir = op_up_dir.join("ops");
    let docker_dir = ops_dir.join("docker");
    let op_monorepo_dir = op_up_dir.join("optimism");
    let op_rs_monorepo_dir = op_up_dir.join("optimism-rs");

    // Files referenced
    let stack_file = op_up_dir.join(".stack");
    let set_timestamp_script = ops_dir.join("devnet_state").join("set_timestamp.sh");

    // ----------------------------------------
    // Create a new op-stack config object from user choices
    // (or load an existing one from the .stack file if it exists)

    let op_stack_config = if stack_file.exists() {
        let existing_stack = utils::read_stack_from_file(&stack_file)?;
        println!("Looks like you've already got an existing op-stack loaded!");

        let use_existing = Confirm::new("Do you want to use the existing stack?")
            .with_default(true)
            .with_help_message(existing_stack.to_string().as_str())
            .prompt()?;

        if use_existing {
            println!("\nGreat! We'll use the existing stack.");
            existing_stack
        } else {
            remove_file(&stack_file)?;
            println!("\nOk, we'll start from scratch then.");
            OpStackConfig::from_user_choices()?
        }
    } else {
        println!("\nWelcome to the interactive op-stack devnet builder!");
        println!("Please select your desired op-stack components:\n");
        OpStackConfig::from_user_choices()?
    };

    // Remember the selected stack for next time
    utils::write_stack_to_file(&stack_file, &op_stack_config)?;

    // Check if the optimism and optimism-rs paths exist in the project root dir.
    // If not, shallow-clone them from github with the `--no-checkout` flag
    if !Path::new(&op_monorepo_dir).exists() {
        println!("Cloning the optimism monorepo from github...");
        utils::git_clone(
            op_up_dir,
            GitCloneMethod::Shallow,
            constants::OP_MONOREPO_URL,
        )?;
    }
    if !Path::new(&op_rs_monorepo_dir).exists() {
        println!("Cloning the optimism-rs monorepo from github...");
        utils::git_clone(
            op_up_dir,
            GitCloneMethod::Shallow,
            constants::OP_RS_MONOREPO_URL,
        )?;
    }

    // ----------------------------------------
    // Based on the components selected, pull the appropriate packages
    // from the op-monorepo and op-rs-monorepo using the `sparse-checkout` git feature

    println!("Pulling the selected components from github...");

    utils::git_sparse_checkout(&op_monorepo_dir, "init", "--cone")?;
    utils::git_sparse_checkout(&op_rs_monorepo_dir, "init", "--cone")?;

    // These components are always pulled as they are required.
    // If in the future there will be more versions of these components,
    // they should become configurable as well, with the rest of the stack
    utils::git_sparse_checkout(&op_monorepo_dir, "add", "op-proposer")?;
    utils::git_sparse_checkout(&op_monorepo_dir, "add", "op-batcher")?;
    utils::git_sparse_checkout(&op_monorepo_dir, "add", "ops-bedrock")?;
    // utils::git_sparse_checkout(&op_monorepo_dir, "add", "op-node")?;
    // utils::git_sparse_checkout(&op_monorepo_dir, "add", "packages/contracts-bedrock")?;

    match op_stack_config.l1_client {
        L1Client::Geth => { /* No extra dependencies needed */ }
        L1Client::Erigon => { /* No extra dependencies needed */ }
    }

    match op_stack_config.l2_client {
        L2Client::OpGeth => { /* No extra dependencies needed */ }
        L2Client::OpErigon => { /* No extra dependencies needed */ }
    }

    match op_stack_config.rollup_client {
        RollupClient::OpNode => utils::git_sparse_checkout(&op_rs_monorepo_dir, "add", "op-node")?,
        RollupClient::Magi => {
            utils::git_clone(op_up_dir, GitCloneMethod::Full, constants::MAGI_REPO_URL)?
        }
    }

    match op_stack_config.challenger_agent {
        ChallengerAgent::OpChallengerGo => {
            utils::git_sparse_checkout(&op_monorepo_dir, "add", "op-challenger")?;
        }
        ChallengerAgent::OpChallengerRust => {
            utils::git_clone(
                op_up_dir,
                GitCloneMethod::Full,
                constants::OP_CHALLENGER_RUST_REPO_URL,
            )?;
        }
    }

    // Finally pull the components added via sparse-checkout
    let op_checkout_components = Command::new("git")
        .arg("checkout")
        .current_dir(&op_monorepo_dir)
        .output()?;
    utils::check_command(
        op_checkout_components,
        &format!("Failed git checkout in {:?}", op_monorepo_dir),
    )?;
    let op_rs_checkout_components = Command::new("git")
        .arg("checkout")
        .current_dir(&op_rs_monorepo_dir)
        .output()?;
    utils::check_command(
        op_rs_checkout_components,
        &format!("Failed git checkout in {:?}", op_rs_monorepo_dir),
    )?;

    // ----------------------------------------
    // Build the devnet

    println!("Components successfully pulled! Building devnet...");

    // Update devnet genesis files with the current timestamps
    utils::make_executable(&set_timestamp_script)?;
    let update_timestamps = Command::new(set_timestamp_script)
        .env("OP_UP_DIR", op_up_dir.to_str().unwrap())
        .output()?;
    utils::check_command(update_timestamps, "Failed to update devnet genesis files")?;

    println!("You may need to enter your password to run docker-compose as sudo.");

    // // Build the docker images
    // let docker_build = Command::new("docker-compose")
    //     .arg("build")
    //     .arg("")
    //     .arg("--progress")
    //     .arg("plain")
    //     .env("DOCKER_BUILDKIT", "1")
    //     .env("L2OO_ADDRESS", constants::L2OO_ADDRESS)
    //     .env("PWD", docker_dir.to_str().unwrap())
    //     .current_dir(&docker_dir)
    //     .output()?;
    // utils::check_command(docker_build, "Failed to build docker images")?;

    // Bring up the L1
    match op_stack_config.l1_client {
        L1Client::Geth => {
            println!("Bringing up geth...");
            let geth_up = Command::new("docker-compose")
                .arg("up")
                .arg("--detach")
                .arg("--no-deps")
                .arg("--build")
                .arg("l1_geth")
                .env("L2OO_ADDRESS", constants::L2OO_ADDRESS)
                .current_dir(&docker_dir)
                .output()?;

            dbg!(geth_up.clone());
            utils::check_command(geth_up, "Failed to bring up geth")?;
        }
        L1Client::Erigon => {
            println!("Bringing up erigon...");
            todo!();
        }
    };

    // Wait for the L1 to be ready
    println!("Waiting for the L1 to be ready...");
    utils::wait_for_response(constants::L1_URL)?;

    Ok(())
}

/// ## OP Stack Config
///
/// Struct to hold the user's choices for the op-stack components
/// that they want to use for their devnet
pub struct OpStackConfig {
    l1_client: L1Client,
    l2_client: L2Client,
    rollup_client: RollupClient,
    challenger_agent: ChallengerAgent,
}

impl Display for OpStackConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "L1 client: {}, L2 client: {}, Rollup client: {}, Challenger: {}",
            self.l1_client, self.l2_client, self.rollup_client, self.challenger_agent,
        )
    }
}

impl OpStackConfig {
    /// ## Generate a new OP Stack config object from user choices
    ///
    /// Prompt the user to select their desired op-stack components
    /// and return a new OpStackConfig struct with their selections
    fn from_user_choices() -> Result<Self> {
        make_selection!(
            l1_client,
            "Which L1 execution client would you like to use?",
            vec![stack::GETH, stack::ERIGON]
        );

        make_selection!(
            l2_client,
            "Which L2 execution client would you like to use?",
            vec![stack::OP_GETH, stack::OP_ERIGON]
        );

        make_selection!(
            rollup_client,
            "Which rollup client would you like to use?",
            vec![stack::OP_NODE, stack::MAGI]
        );

        make_selection!(
            challenger_agent,
            "Which challenger agent would you like to use?",
            vec![stack::OP_CHALLENGER_GO, stack::OP_CHALLENGER_RUST]
        );

        println!("\nNice choice! You've got great taste ✨");

        Ok(OpStackConfig {
            l1_client: l1_client.parse()?,
            l2_client: l2_client.parse()?,
            rollup_client: rollup_client.parse()?,
            challenger_agent: challenger_agent.parse()?,
        })
    }
}
