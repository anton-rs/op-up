use std::{fs::create_dir_all, path::Path};

use eyre::Result;
use inquire::Confirm;

mod items;
mod utils;

fn main() -> Result<()> {
    println!("\nWelcome to the op-up CLI!");
    println!("(This is a work in progress, some things may not work as expected)");
    println!("---------------------------------------------------\n");

    create_dir_all("../.devnet")?;

    if Path::new("../.stack").exists() {
        println!("Looks like you've already got an existing op-stack loaded!");

        match Confirm::new("Do you want to use the existing stack?")
            .with_default(true)
            .with_help_message(format!("Your stack: {:?}", utils::read_stack_from_file()?).as_str())
            .prompt()?
        {
            true => {
                println!("\nGreat! We'll use the existing stack.");
                println!("---------------------------------------------------\n");

                return Ok(()); // TODO
            }
            false => {
                println!("\nOk, we'll start from scratch then.");
                println!("---------------------------------------------------\n");
            }
        }
    }

    // check if the optimism and optimism-rs paths exist in the root directory
    // if not, clone them from github with the --no-checkout flag
    if !Path::new("../optimism").exists() {
        println!("Cloning the optimism monorepo from github...");
        git_clone!(
            "--no-checkout",
            "git@github.com:ethereum-optimism/optimism.git"
        );
    }
    if !Path::new("../optimism-rs").exists() {
        println!("Cloning the optimism-rs monorepo from github...");
        git_clone!("--no-checkout", "git@github.com:refcell/optimism-rs.git");
    }

    make_selection!(
        l1_client_choice,
        "Which L1 execution client would you like to use?",
        vec![items::GETH, items::ERIGON]
    );

    make_selection!(
        l2_client_choice,
        "Which L2 execution client would you like to use?",
        vec![items::OP_GETH, items::OP_ERIGON]
    );

    make_selection!(
        rollup_client_choice,
        "Which rollup client would you like to use?",
        vec![items::OP_NODE, items::MAGI]
    );

    make_selection!(
        challenger_agent,
        "Which challenger agent would you like to use?",
        vec![items::OP_CHALLENGER_GO, items::OP_CHALLENGER_RUST]
    );

    utils::write_stack_to_file(vec![
        l1_client_choice,
        l2_client_choice,
        rollup_client_choice,
        challenger_agent,
    ])?;

    println!("Nice choice! You've got great taste in this stuff ✨");
    println!("---------------------------------------------------\n");

    Ok(())
}
