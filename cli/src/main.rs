use std::{env::current_dir, fmt::Display, fs::remove_file, path::Path, process::Command};

use eyre::{eyre, Result};
use inquire::Confirm;

use crate::stack::{ChallengerAgent, L1Client, L2Client, RollupClient};

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
    let docker_dir = op_up_dir.join("docker");
    let op_monorepo_dir = op_up_dir.join("optimism");
    let op_rs_monorepo_dir = op_up_dir.join("optimism-rs");

    // Files referenced
    let stack_file = op_up_dir.join(".stack");
    let devnet_up_script = docker_dir.join("devnet-up.sh");

    // ----------------------------------------
    // Create a new op-stack config object from user choices
    // (or load an existing one from the .stack file if it exists)

    let stack = if stack_file.exists() {
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
    utils::write_stack_to_file(&stack_file, &stack)?;

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

    // Run the main orchestration script
    utils::make_executable(&devnet_up_script)?;
    let devnet_up = Command::new(devnet_up_script)
        .env("L1_CLIENT_CHOICE", stack.l1_client.to_string())
        .env("L2_CLIENT_CHOICE", stack.l2_client.to_string())
        .env("ROLLUP_CLIENT_CHOICE", stack.rollup_client.to_string())
        .env("CHALLENGER_AGENT_CHOICE", stack.challenger.to_string())
        .current_dir(&docker_dir)
        .output()?;

    dbg!(devnet_up.clone()); // TODO: remove

    utils::check_command(devnet_up, "Failed to build devnet")?;

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
    challenger: ChallengerAgent,
}

impl Display for OpStackConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "L1 client: {}, L2 client: {}, Rollup client: {}, Challenger: {}",
            self.l1_client, self.l2_client, self.rollup_client, self.challenger,
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
            challenger,
            "Which challenger agent would you like to use?",
            vec![stack::OP_CHALLENGER_GO, stack::OP_CHALLENGER_RUST]
        );

        println!("\nNice choice! You've got great taste ✨");

        Ok(OpStackConfig {
            l1_client: l1_client.parse()?,
            l2_client: l2_client.parse()?,
            rollup_client: rollup_client.parse()?,
            challenger: challenger.parse()?,
        })
    }
}
