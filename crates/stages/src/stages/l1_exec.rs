use eyre::Result;
use std::process::Command;

/// L1 Execution Client Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Executor {
    l1_client: String,
}

impl crate::Stage for Executor {
    /// Executes the L1 Executor Stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing l1 execution client stage");

        // todo: this should be replaced with running the docker container inline through
        // the op-composer crate anyways so we won't need the docker directory at all.
        let proj_root = project_root::get_project_root()?;
        let docker_dir = proj_root.as_path().join("docker");

        let exec = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "l1"])
            .env("PWD", &docker_dir)
            .env("L1_CLIENT_CHOICE", &self.l1_client)
            .current_dir(docker_dir)
            .output()?;

        if !exec.status.success() {
            eyre::bail!(
                "failed to start l1 execution client: {}",
                String::from_utf8_lossy(&exec.stderr)
            );
        }

        // todo: use a configured port here
        crate::net::wait_up(op_config::L1_PORT, 10, 1)?;

        // todo: do we need to do this???
        // block entire thread, because we don't have tokio, or any similar dependency
        std::thread::sleep(std::time::Duration::from_secs(10));

        Ok(())
    }
}

impl Executor {
    /// Creates a new stage.
    pub fn new(l1_client: String) -> Self {
        Self { l1_client }
    }
}
