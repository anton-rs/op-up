use eyre::Result;
use maplit::hashmap;
use op_primitives::L1Client;
use std::sync::Arc;

use async_trait::async_trait;

use op_composer::{bind_host_port, Composer, Config, CreateVolumeOptions, HostConfig};
use op_primitives::Artifacts;

/// L1 Execution Client Stage
#[derive(Debug)]
pub struct Executor {
    l1_port: Option<u16>,
    l1_client: L1Client,
    l1_exec: Arc<Composer>,
    artifacts: Arc<Artifacts>,
}

#[async_trait]
impl crate::Stage for Executor {
    /// Executes the L1 Executor Stage.
    async fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing l1 execution client stage");

        match self.l1_client {
            L1Client::Geth => self.start_geth().await?,
            _ => unimplemented!("l1 client not implemented: {}", self.l1_client),
        }

        Ok(())
    }
}

impl Executor {
    /// Creates a new stage.
    pub fn new(
        l1_port: Option<u16>,
        l1_client: L1Client,
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

    /// Starts Geth in a docker container.
    pub async fn start_geth(&self) -> Result<()> {
        let image_name = "opup-geth".to_string();
        let working_dir = project_root::get_project_root()?.join("docker");
        let l1_genesis = self.artifacts.l1_genesis();
        let l1_genesis = l1_genesis.to_string_lossy();
        let jwt_secret = self.artifacts.jwt_secret();
        let jwt_secret = jwt_secret.to_string_lossy();

        let dockerfile = r#"
            FROM ethereum/client-go:v1.12.2
            RUN apk add --no-cache jq
            COPY geth-entrypoint.sh /geth-entrypoint.sh
            VOLUME ["/db"]
            ENTRYPOINT ["/bin/sh", "/geth-entrypoint.sh"]
        "#;

        let geth_entrypoint = std::fs::read(working_dir.join("geth-entrypoint.sh"))?;
        let build_context_files = [("geth-entrypoint.sh", geth_entrypoint.as_slice())];
        self.l1_exec
            .build_image(&image_name, dockerfile, &build_context_files)
            .await?;

        let l1_data_volume = CreateVolumeOptions {
            name: "l1_data",
            driver: "local",
            ..Default::default()
        };
        self.l1_exec.create_volume(l1_data_volume).await?;

        let config = Config {
            image: Some(image_name),
            working_dir: Some(working_dir.to_string_lossy().to_string()),
            exposed_ports: Some(hashmap! {
                "8545".to_string() => hashmap! {},
                "8546".to_string() => hashmap! {},
                "6060".to_string() => hashmap! {},
            }),
            // TODO: add env vars to change values in entrypoint script
            host_config: Some(HostConfig {
                port_bindings: Some(hashmap! {
                    "8545".to_string() => bind_host_port(8545),
                    "8546".to_string() => bind_host_port(8546),
                    "6060".to_string() => bind_host_port(7060), // TODO: double check this port
                }),
                binds: Some(vec![
                    "l1_data:/db".to_string(),
                    format!("{}:/genesis.json", l1_genesis),
                    format!("{}:/config/test-jwt-secret.txt", jwt_secret),
                ]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let container_id = self
            .l1_exec
            .create_container(&self.l1_client.to_string(), config, true)
            .await?
            .id;

        let all_containers = self.l1_exec.list_containers(None).await?;
        tracing::info!(target: "stages", "all containers: {:?}", all_containers);

        tracing::info!(target: "stages", "l1 container created: {}", container_id);

        self.l1_exec.start_container(&container_id).await?;

        let l1_port = self.l1_port.unwrap_or(op_config::L1_PORT);
        crate::net::wait_up(l1_port, 10, 3)?;
        tracing::info!(target: "stages", "l1 container started on port: {}", l1_port);

        // todo: do we need to do block here
        // can we wait for the l1 client to be ready by polling?
        std::thread::sleep(std::time::Duration::from_secs(10));

        Ok(())
    }
}
