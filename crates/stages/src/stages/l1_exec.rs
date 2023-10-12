use eyre::Result;
use std::process::Command;

/// L1 Execution Client Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Executor {
    l1_port: Option<u16>,
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

        let l1_port = self.l1_port.unwrap_or(op_config::L1_PORT);
        crate::net::wait_up(l1_port, 10, 1)?;

        // todo: do we need to do block here
        // can we wait for the l1 client to be ready by polling?
        std::thread::sleep(std::time::Duration::from_secs(10));

        Ok(())
    }
}

impl Executor {
    /// Creates a new stage.
    pub fn new(l1_port: Option<u16>, l1_client: String) -> Self {
        Self { l1_port, l1_client }
    }
}
