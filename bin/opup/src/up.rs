use clap::Args;
use eyre::Result;
use std::path::{Path, PathBuf};
use tracing::instrument;

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

    /// Internal async executor.
    async fn execute(&self) -> Result<()> {
        tracing::info!("bootstrapping op stack");

        crate::deps::DependencyManager::sync().await?;

        if self.devnet {
            tracing::info!("Building default devnet stack");
            let config = Config::default().force_overwrites(self.force);
            return Stages::from(config).execute().await;
        }

        // Get the directory of the config file if it exists.
        let config_dir = self.config.as_ref().and_then(|p| p.parent());
        let config_dir = config_dir.unwrap_or_else(|| Path::new("."));
        tracing::info!("Using config directory: {:?}", config_dir);

        // Load the config file from the parsed path.
        let config = Config::load_with_root(config_dir).force_overwrites(self.force);
        tracing::info!("Built config, executing stages");
        Stages::from(config).execute().await
    }

    /// Entrypoint
    #[instrument(name = "up", target = "run")]
    pub fn run(&self) -> Result<()> {
        crate::runner::run_until_ctrl_c(async { self.execute().await })
    }
}
