use clap::{ArgAction, Parser};
use eyre::Result;

use crate::etc::telemetry;
use crate::stack;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Verbosity level (0-4)
    #[arg(long, short, action = ArgAction::Count, default_value = "2")]
    v: u8,
}

pub fn run() -> Result<()> {
    let Args { v } = Args::parse();

    telemetry::init_tracing_subscriber(v)?;

    tracing::info!(target: "opup", "bootstrapping op stack");

    stack::run()?;

    Ok(())
}
