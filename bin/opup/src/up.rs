use clap::Args;
use eyre::Result;
use std::path::{Path, PathBuf};

use op_config::Config;
use op_stages::Stages;

/// The Up CLI Subcommand.
#[derive(Debug, Args)]
pub struct UpCommand {
    /// An optional path to a stack config file.
    #[arg(long, short)]
    pub config: Option<PathBuf>,

    /// Whether to build a hard-coded default devnet stack, ignoring the config file.
    #[arg(long, short)]
    pub devnet: bool,

    /// Force enables op-up to enter overwrite mode.
    ///
    /// It enables overwriting of persistant artifacts from previous runs,
    /// for example, git repository clones.
    #[arg(long, short)]
    pub force: bool,
}

impl UpCommand {
    /// Create a new Up CLI Subcommand.
    pub fn new(config: Option<PathBuf>, devnet: bool) -> Self {
        Self {
            config,
            devnet,
            force: false,
        }
    }

    /// Run the Up CLI Subcommand.
    pub fn run(&self) -> Result<()> {
        crate::runner::run_until_ctrl_c(async {
            tracing::info!(target: "cli", "bootstrapping op stack");

            // todo get the force arg and pass it into the stages pipeline
            // should the stack config be transformed to include this and
            // other flags?

            if self.devnet {
                tracing::info!(target: "cli", "Building default devnet stack");
                Stages::from(Config::default().force_overwrites(self.force))
                    .execute()
                    .await
            } else {
                // Get the directory of the config file if it exists.
                let config_dir = self.config.as_ref().and_then(|p| p.parent());
                let config_dir = config_dir.unwrap_or_else(|| Path::new("."));

                // Build a config from the parsed config directory.
                tracing::info!(target: "cli", "Loading op-stack config from {:?}", config_dir);
                let stack = Config::load_with_root(config_dir).force_overwrites(self.force);

                tracing::info!(target: "cli", "Stack: {:#?}", stack);

                Stages::from(stack).execute().await
            }
        })
    }
}
