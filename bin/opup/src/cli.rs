use clap::{ArgAction, Parser};
use eyre::Result;
use std::path::PathBuf;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Verbosity level (0-4)
    #[arg(long, short, action = ArgAction::Count, default_value = "2")]
    v: u8,
    /// An optional path to a stack config file.
    #[arg(long, short)]
    config: Option<PathBuf>,
}

pub fn run() -> Result<()> {
    let Args { v, config } = Args::parse();

    crate::telemetry::init_tracing_subscriber(v)?;

    crate::banners::banner()?;

    // todo: switch on subcommands
    // default should be to run the devnet stack
    // should also allow nuking the devnet
    // and stopping an already running devnet

    crate::stack::Stack::new(config).run()
}
