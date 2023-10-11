use std::path::PathBuf;
use std::process::Command;

use eyre::Result;

/// L1 Execution Client Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Executor {
    /// The path to the Dockerfile directory.
    pub docker_dir: Option<PathBuf>,
    /// The l1 client choice.
    pub l1_client: String,
}

impl crate::Stage for Executor {
    /// Executes the L1 Executor Stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing l1 execution client stage");

        let docker_dir = self
            .docker_dir
            .as_ref()
            .map(|p| p.to_str())
            .flatten()
            .ok_or(eyre::eyre!("missing dockerfile directory"))?;

        let exec = Command::new("docker-compose")
            .args(["up", "-d", "--no-deps", "--build", "l1"])
            .env("PWD", docker_dir)
            .env("L1_CLIENT_CHOICE", &self.l1_client)
            .current_dir(&docker_dir)
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
    pub fn new(docker_dir: Option<PathBuf>, l1_client: String) -> Self {
        Self {
            docker_dir: Some(docker_dir.unwrap_or(Executor::get_docker_dir_unsafe())),
            l1_client,
        }
    }

    /// Returns a [PathBuf] for the Dockerfile directory.
    ///
    /// # Panics
    ///
    /// Panics if the [project_root::get_project_root] function call fails to return a valid
    /// project root [PathBuf].
    pub fn get_docker_dir_unsafe() -> PathBuf {
        let proj_root = project_root::get_project_root().expect("Failed to get project root");
        let op_up_dir = proj_root.as_path();
        op_up_dir.join("docker")
    }
}
