use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

use eyre::{eyre, Result};

use crate::{make_selection, stack};

pub mod l1_client;
pub use l1_client::{L1Client, ERIGON, GETH};

pub mod l2_client;
pub use l2_client::{L2Client, OP_ERIGON, OP_GETH};

pub mod rollup_client;
pub use rollup_client::{RollupClient, MAGI, OP_NODE};

pub mod challenger_agent;
pub use challenger_agent::{ChallengerAgent, OP_CHALLENGER_GO, OP_CHALLENGER_RUST};

/// ## OP Stack Config
///
/// Struct to hold the user's choices for the op-stack components
/// that they want to use for their devnet
pub struct OpStackConfig {
    pub l1_client: L1Client,
    pub l2_client: L2Client,
    pub rollup_client: RollupClient,
    pub challenger: ChallengerAgent,
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
    pub fn from_user_choices() -> Result<Self> {
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

pub fn read_from_file(file: &PathBuf) -> Result<OpStackConfig> {
    let file = File::open(file)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap_or_default()).collect();

    Ok(OpStackConfig {
        l1_client: lines
            .get(0)
            .ok_or(eyre!("expected l1_client at line 1"))?
            .to_string()
            .parse()?,
        l2_client: lines
            .get(1)
            .ok_or(eyre!("expected l2_client at line 2"))?
            .to_string()
            .parse()?,
        rollup_client: lines
            .get(2)
            .ok_or(eyre!("expected rollup_client at line 3"))?
            .to_string()
            .parse()?,
        challenger: lines
            .get(3)
            .ok_or(eyre!("expected challenger_agent at line 4"))?
            .to_string()
            .parse()?,
    })
}

pub fn write_to_file(file: &PathBuf, stack: &OpStackConfig) -> Result<()> {
    let file = File::create(file)?;
    let mut writer = BufWriter::new(file);

    let mut line = String::new();
    line.push_str(&stack.l1_client.to_string());
    line.push('\n');
    writer.write_all(line.as_bytes())?;

    let mut line = String::new();
    line.push_str(&stack.l2_client.to_string());
    line.push('\n');
    writer.write_all(line.as_bytes())?;

    let mut line = String::new();
    line.push_str(&stack.rollup_client.to_string());
    line.push('\n');
    writer.write_all(line.as_bytes())?;

    let mut line = String::new();
    line.push_str(&stack.challenger.to_string());
    line.push('\n');
    writer.write_all(line.as_bytes())?;

    Ok(())
}
