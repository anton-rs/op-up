use async_trait::async_trait;
use eyre::Result;
use std::process::Command;

/// Layer 2 Execution Client Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Executor {
    l2_port: Option<u16>,
    l2_client: String,
}

#[async_trait]
impl crate::Stage for Executor {
    /// Executes the L2 Executor Stage.
    async fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing l2 execution client stage");

        // todo: this should be replaced with running the docker container inline through
        // the op-composer crate anyways so we won't need the docker directory at all.
        let proj_root = project_root::get_project_root()?;
        let docker_dir = proj_root.as_path().join("docker");

        let start_l2 = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "l2"])
            .env("PWD", &docker_dir)
            .env("L2_CLIENT_CHOICE", &self.l2_client)
            .current_dir(docker_dir)
            .output()?;

        if !start_l2.status.success() {
            eyre::bail!(
                "failed to start l2 execution client: {}",
                String::from_utf8_lossy(&start_l2.stderr)
            );
        }

        let l2_port = self.l2_port.unwrap_or(op_config::L2_PORT);
        crate::net::wait_up(l2_port, 10, 1)
    }
}

impl Executor {
    /// Creates a new stage.
    pub fn new(l2_port: Option<u16>, l2_client: String) -> Self {
        Self { l2_port, l2_client }
    }
}
