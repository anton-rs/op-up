use clap::{ArgAction, Parser, Subcommand};
use eyre::Result;
use std::path::PathBuf;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Verbosity level (0-4)
    #[arg(long, short, action = ArgAction::Count, default_value = "2")]
    v: u8,

    /// The subcommand to run
    #[clap(subcommand)]
    pub command: Option<Command>,

    /// An optional path to a stack config file.
    #[arg(long, short)]
    config: Option<PathBuf>,
}

/// Possible CLI subcommands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Build and run the devnet stack
    Up,
    /// Bring the devnet stack down
    Down,
    /// Nuke the devnet stack
    Nuke,
    /// Clean all stack artifacts
    Clean,
}

pub fn run() -> Result<()> {
    let Args { v, config, command } = Args::parse();

    crate::telemetry::init_tracing_subscriber(v)?;

    crate::banners::banner()?;

    match command {
        Some(Command::Up) | None => crate::stack::Stack::new(config).run()?,
        Some(Command::Down) => unimplemented!("down command not yet implemented"),
        Some(Command::Nuke) => unimplemented!("nuke command not yet implemented"),
        Some(Command::Clean) => unimplemented!("clean command not yet implemented"),
    }

    Ok(())
}
