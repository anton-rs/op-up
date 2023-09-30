use anyhow::{anyhow, Result};
use clap::{ArgAction, Parser};
use std::{error::Error, sync::Arc};
use tracing::Level;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Verbosity level (0-4)
    #[arg(long, short, action = ArgAction::Count, default_value = "2")]
    v: u8,
}

pub fn main() -> Result<()> {
    let Args { v } = Args::parse();

    opup::telemetry::init_tracing_subscriber(v)?;

    tracing::info!(target: "opup", "bootstrapping op stack");

    opup::boot::run()?;

    Ok(())
}
