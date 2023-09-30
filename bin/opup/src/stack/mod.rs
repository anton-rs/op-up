use anyhow::Result;
use bollard::Docker;

use crate::etc::runner;

/// Spin up the stack.
pub fn run() -> Result<()> {
    runner::run_until_ctrl_c(async {
        let docker = Docker::connect_with_local_defaults()?;
        let version = docker.version().await?;
        tracing::info!(target: "opup", "docker version: {:?}", version);
        Ok(())
    })
}
