use std::{env::current_dir, fmt::Display, fs::remove_file};

use eyre::{eyre, Result};
use inquire::Confirm;

mod constants;
mod utils;

fn main() -> Result<()> {
    println!("\nWelcome to the op-up CLI!");
    println!("(This is a work in progress, some things may not work as expected)");
    println!("---------------------------------------------------\n");

    let cwd = current_dir()?;
    let op_up_dir = cwd.parent().ok_or(eyre!("Failed to get project root"))?;

    // Files referenced
    let stack_file = op_up_dir.join(".stack");

    // Directories referenced
    let ops_dir = op_up_dir.join("ops");
    let op_monorepo_dir = op_up_dir.join("optimism");
    let op_rs_monorepo_dir = op_up_dir.join("optimism-rs");

    let op_stack = if stack_file.exists() {
        let existing_stack = utils::read_stack_from_file(&stack_file)?;
        println!("Looks like you've already got an existing op-stack loaded!");

        let use_existing = Confirm::new("Do you want to use the existing stack?")
            .with_default(true)
            .with_help_message(existing_stack.to_string().as_str())
            .prompt()?;

        if use_existing {
            println!("\nGreat! We'll use the existing stack.");
            println!("---------------------------------------------------\n");
            existing_stack
        } else {
            remove_file(&stack_file)?;
            select_stack()?
        }
    } else {
        select_stack()?
    };

    // check if the optimism and optimism-rs paths exist in the root directory
    // if not, clone them from github with the --no-checkout flag
    if !op_monorepo_dir.exists() {
        println!("Cloning the optimism monorepo from github...");
        git_clone!("--no-checkout", constants::OP_MONOREPO_URL);
    }
    if !op_rs_monorepo_dir.exists() {
        println!("Cloning the optimism-rs monorepo from github...");
        git_clone!("--no-checkout", constants::OP_RS_MONOREPO_URL);
    }

    utils::write_stack_to_file(&stack_file, op_stack)?;

    // based on the components selected, pull the appropriate packages
    // from the monorepos and build them
    // TODO

    // -------------------------------------------------------------
    println!("Components successfully pulled! Building devnet...");
    println!("---------------------------------------------------\n");

    // Update devnet genesis files with the current timestamps
    std::process::Command::new("../ops/devnet_state/set_timestamp.sh")
        .current_dir(&ops_dir)
        .output()
        .expect("Failed to update genesis files with current timestamps");

    Ok(())
}

pub struct OpStackConfig {
    l1_client: String,
    l2_client: String,
    rollup_client: String,
    challenger_agent: String,
}

impl Display for OpStackConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "L1 client: {}, L2 client: {}, Rollup client: {}, Challenger agent: {}",
            self.l1_client, self.l2_client, self.rollup_client, self.challenger_agent,
        )
    }
}

fn select_stack() -> Result<OpStackConfig> {
    println!("\nOk, we'll start from scratch then.");
    println!("---------------------------------------------------\n");

    make_selection!(
        l1_client,
        "Which L1 execution client would you like to use?",
        vec![constants::GETH, constants::ERIGON]
    );

    make_selection!(
        l2_client,
        "Which L2 execution client would you like to use?",
        vec![constants::OP_GETH, constants::OP_ERIGON]
    );

    make_selection!(
        rollup_client,
        "Which rollup client would you like to use?",
        vec![constants::OP_NODE, constants::MAGI]
    );

    make_selection!(
        challenger_agent,
        "Which challenger agent would you like to use?",
        vec![constants::OP_CHALLENGER_GO, constants::OP_CHALLENGER_RUST]
    );

    println!("Nice choice! You've got great taste in this stuff ✨");
    println!("---------------------------------------------------\n");

    Ok(OpStackConfig {
        l1_client,
        l2_client,
        rollup_client,
        challenger_agent,
    })
}
