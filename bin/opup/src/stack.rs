use bollard::Docker;
use eyre::Result;
use std::path::{Path, PathBuf};

use op_config::Config;
use op_stages::Stages;

/// The Stack CLI Command.
#[derive(Debug)]
pub struct Stack {
    /// An optional path to a stack config file.
    pub config: Option<PathBuf>,
}

impl Stack {
    /// Create a new Stack CLI Command.
    pub fn new(config: Option<PathBuf>) -> Self {
        Self { config }
    }

    /// Run the Stack CLI Command.
    pub fn run(&self) -> Result<()> {
        crate::runner::run_until_ctrl_c(async {
            tracing::info!(target: "cli", "bootstrapping op stack");

            // todo: remove this once we have a proper stage docker component
            //       for now, this placeholds use of [bollard].
            let docker = Docker::connect_with_local_defaults()?;
            let version = docker.version().await?;
            tracing::info!(target: "cli", "docker version: {:?}", version);

            // Get the directory of the config file if it exists.
            let config_dir = self.config.as_ref().and_then(|p| p.parent());
            let config_dir = config_dir.unwrap_or_else(|| Path::new("."));

            // Build a config from the parsed config directory.
            tracing::info!(target: "cli", "Loading op-stack config from {:?}", config_dir);
            let stack = Config::load_with_root(config_dir);

            Stages::from(stack).execute().await
        })
    }
}
