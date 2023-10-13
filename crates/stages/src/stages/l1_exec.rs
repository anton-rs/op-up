use eyre::Result;
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use op_composer::Composer;
use op_composer::ContainerConfig;
use op_primitives::Artifacts;

/// L1 Execution Client Stage
#[derive(Debug)]
pub struct Executor {
    l1_port: Option<u16>,
    l1_client: String,
    l1_exec: Arc<Composer>,
    artifacts: Arc<Artifacts>,
}

#[async_trait]
impl crate::Stage for Executor {
    /// Executes the L1 Executor Stage.
    async fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing l1 execution client stage");

        let working_dir = project_root::get_project_root()?.join("docker");
        let l1_genesis = self.artifacts.l1_genesis();
        let jwt_secret = self.artifacts.jwt_secret();
        let config = ContainerConfig {
            cmd: Some(vec![
                "/bin/sh".to_string(),
                "geth-entrypoint.sh".to_string(),
            ]),
            working_dir: Some(working_dir.to_string_lossy().to_string()),
            volumes: Some(HashMap::from([
                ("l1_data:/db".to_string(), HashMap::new()),
                (
                    format!("{}:/genesis.json", l1_genesis.to_string_lossy()),
                    HashMap::new(),
                ),
                (
                    format!(
                        "{}:/config/test-jwt-secret.txt",
                        jwt_secret.to_string_lossy()
                    ),
                    HashMap::new(),
                ),
            ])),
            exposed_ports: Some(HashMap::from([
                ("8545:8545".to_string(), HashMap::new()),
                ("8546:8546".to_string(), HashMap::new()),
                ("7060:6060".to_string(), HashMap::new()),
            ])),
            ..Default::default()
        };

        let response = self
            .l1_exec
            .create_container(&self.l1_client, config)
            .await?;
        tracing::info!(target: "stages", "l1 container created: {:?}", response);

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
    pub fn new(
        l1_port: Option<u16>,
        l1_client: String,
        l1_exec: Arc<Composer>,
        artifacts: Arc<Artifacts>,
    ) -> Self {
        Self {
            l1_port,
            l1_client,
            l1_exec,
            artifacts,
        }
    }
}
