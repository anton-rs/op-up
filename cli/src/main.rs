use std::{env::current_dir, fmt::Display, fs::remove_file, path::Path};

use eyre::{bail, eyre, Result};
use inquire::Confirm;

mod constants;
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
    let op_monorepo_dir = op_up_dir.join("optimism");
    let op_rs_monorepo_dir = op_up_dir.join("optimism-rs");

    // Files referenced
    let stack_file = op_up_dir.join(".stack");
    let set_timestamp_script = ops_dir.join("devnet_state").join("set_timestamp.sh");

    let op_stack = if stack_file.exists() {
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
            select_stack()?
        }
    } else {
        println!("\nWelcome to the interactive op-stack devnet builder!");
        println!("Please select your desired op-stack components:\n");
        select_stack()?
    };

    // Remember the selected stack for next time
    utils::write_stack_to_file(&stack_file, &op_stack)?;

    // Check if the optimism and optimism-rs paths exist in the root directory
    // if not, clone them from github with the --no-checkout flag
    if !Path::new(&op_monorepo_dir).exists() {
        println!("Cloning the optimism monorepo from github...");
        git_clone!(op_up_dir, "--no-checkout", constants::OP_MONOREPO_URL);
    }
    if !Path::new(&op_rs_monorepo_dir).exists() {
        println!("Cloning the optimism-rs monorepo from github...");
        git_clone!(op_up_dir, "--no-checkout", constants::OP_RS_MONOREPO_URL);
    }

    // ----------------------------------------
    // Based on the components selected, pull the appropriate packages
    // from the op-monorepo and op-rs-monorepo using the sparse-checkout git feature
    println!("Pulling the selected components from github...");

    git_sparse_checkout!(&op_monorepo_dir, "init", "--cone");
    git_sparse_checkout!(&op_rs_monorepo_dir, "init", "--cone");

    // These components are always pulled as they are required.
    // If in the future there will be more versions of these components,
    // they should become configurable as well, with the rest of the stack
    git_sparse_checkout!(&op_monorepo_dir, "set", "packages/contracts-bedrock"); // todo check if needed
    git_sparse_checkout!(&op_monorepo_dir, "set", "op-node");
    git_sparse_checkout!(&op_monorepo_dir, "set", "op-proposer");
    git_sparse_checkout!(&op_monorepo_dir, "set", "op-batcher");
    git_sparse_checkout!(&op_monorepo_dir, "set", "ops-bedrock");

    match op_stack.l1_client.as_str() {
        constants::GETH => {}
        constants::ERIGON => {}
        _ => bail!("Invalid L1 client found in stack"),
    }

    match op_stack.l2_client.as_str() {
        constants::OP_GETH => {}
        constants::OP_ERIGON => {}
        _ => bail!("Invalid L2 client found in stack"),
    }

    match op_stack.rollup_client.as_str() {
        constants::OP_NODE => {}
        constants::MAGI => {}
        _ => bail!("Invalid rollup client found in stack"),
    }

    match op_stack.challenger_agent.as_str() {
        constants::OP_CHALLENGER_GO => {}
        constants::OP_CHALLENGER_RUST => {}
        _ => bail!("Invalid challenger agent found in stack"),
    }

    println!("Components successfully pulled! Building devnet...");

    // Update devnet genesis files with the current timestamps
    make_executable!(set_timestamp_script);
    let update_timestamps = std::process::Command::new(set_timestamp_script)
        .env("OP_UP_DIR", op_up_dir.to_str().unwrap())
        .output()?;

    if !update_timestamps.status.success() {
        bail!(
            "Failed to update devnet genesis files with current timestamps: {}",
            String::from_utf8_lossy(&update_timestamps.stderr)
        );
    }

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
            "L1 client: {}, L2 client: {}, Rollup client: {}, Challenger: {}",
            self.l1_client, self.l2_client, self.rollup_client, self.challenger_agent,
        )
    }
}

fn select_stack() -> Result<OpStackConfig> {
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

    println!("\nNice choice! You've got great taste in this stuff ✨");

    Ok(OpStackConfig {
        l1_client,
        l2_client,
        rollup_client,
        challenger_agent,
    })
}
