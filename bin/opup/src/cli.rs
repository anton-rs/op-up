use clap::{ArgAction, Parser, Subcommand};
use eyre::Result;

use crate::up::UpCommand;

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
}

/// Possible CLI subcommands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Build and run the devnet stack
    Up(UpCommand),
    /// Bring the devnet stack down
    Down,
    /// Nuke the devnet stack
    Nuke,
    /// Clean all stack artifacts
    Clean,
    /// List op-up docker containers
    List,
    /// Watch the devnet stack components.
    /// This will output a refreshed view of the stack components
    /// as they come online.
    Watch,
    /// Install Dependencies
    Deps,
}

pub fn run() -> Result<()> {
    let Args { v, command } = Args::parse();

    crate::telemetry::init_tracing_subscriber(v)?;

    // Dispatch on the specified subcommand,
    // running the `up` subcommand by default.
    match command {
        None => UpCommand::new(None, false).run(),
        Some(command) => match command {
            Command::Up(up_command) => up_command.run(),
            Command::List => crate::list::run(),
            Command::Watch => crate::watch::run(),
            Command::Down => unimplemented!("down command not yet implemented"),
            Command::Nuke => unimplemented!("nuke command not yet implemented"),
            Command::Clean => unimplemented!("clean command not yet implemented"),
            Command::Deps => {
                tracing::info!(target: "opup", "Installing dependencies...");
                crate::runner::run_until_ctrl_c(async {
                    crate::deps::DependencyManager::sync().await
                })?;
                tracing::info!(target: "opup", "Dependencies installed.");
                Ok(())
            }
        },
    }
}
