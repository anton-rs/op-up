use async_trait::async_trait;
use eyre::Result;
use maplit::hashmap;
use op_composer::{
    bind_host_port, BuildContext, Composer, Config, CreateVolumeOptions, HostConfig,
};
use op_primitives::{Artifacts, L2Client};
use std::sync::Arc;

/// Layer 2 Execution Client Stage
#[derive(Debug)]
pub struct Executor {
    l2_port: Option<u16>,
    l2_client: L2Client,
    l2_exec: Arc<Composer>,
    artifacts: Arc<Artifacts>,
}

#[async_trait]
impl crate::Stage for Executor {
    /// Executes the L2 Executor Stage.
    async fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing l2 execution client stage");

        match self.l2_client {
            L2Client::OpGeth => self.start_op_geth().await,
            _ => unimplemented!("l2 execution client not implemented: {}", self.l2_client),
        }
    }
}

impl Executor {
    /// Creates a new stage.
    pub fn new(
        l2_port: Option<u16>,
        l2_client: L2Client,
        l2_exec: Arc<Composer>,
        artifacts: Arc<Artifacts>,
    ) -> Self {
        Self {
            l2_port,
            l2_client,
            l2_exec,
            artifacts,
        }
    }

    /// Starts Op-Geth in a Docker container.
    pub async fn start_op_geth(&self) -> Result<()> {
        let image_name = "opup-l2-geth".to_string();
        let working_dir = project_root::get_project_root()?.join("docker");
        let l2_genesis = self.artifacts.l2_genesis();
        let l2_genesis = l2_genesis.to_string_lossy();
        let jwt_secret = self.artifacts.jwt_secret();
        let jwt_secret = jwt_secret.to_string_lossy();

        let dockerfile = r#"
            FROM us-docker.pkg.dev/oplabs-tools-artifacts/images/op-geth:optimism
            RUN apk add --no-cache jq
            COPY geth-entrypoint.sh /geth-entrypoint.sh
            VOLUME ["/db"]
            ENTRYPOINT ["/bin/sh", "/geth-entrypoint.sh"]
        "#;

        let context = BuildContext::from_dockerfile(dockerfile)
            .add_file(working_dir.join("geth-entrypoint.sh"), "geth-entrypoint.sh");
        self.l2_exec.build_image(&image_name, context).await?;

        let l2_data_volume = CreateVolumeOptions {
            name: "l2_data",
            driver: "local",
            ..Default::default()
        };
        self.l2_exec.create_volume(l2_data_volume).await?;

        let config = Config {
            image: Some(image_name),
            working_dir: Some(working_dir.to_string_lossy().to_string()),
            exposed_ports: Some(hashmap! {
                "8545".to_string() => hashmap!{},
                "6060".to_string() => hashmap!{},
            }),
            host_config: Some(HostConfig {
                port_bindings: Some(hashmap! {
                    "8545".to_string() => bind_host_port(9545),
                    "6060".to_string() => bind_host_port(8060),
                }),
                binds: Some(vec![
                    "l2_data:/db".to_string(),
                    format!("{}:/genesis.json", l2_genesis),
                    format!("{}:/config/test-jwt-secret.txt", jwt_secret),
                ]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let container_id = self
            .l2_exec
            .create_container("opup-l2", config, true)
            .await?
            .id;
        tracing::info!(target: "stages", "l2 container created: {}", container_id);

        self.l2_exec.start_container(&container_id).await?;

        let l2_port = self.l2_port.unwrap_or(op_config::L2_PORT);
        crate::net::wait_up(l2_port, 10, 1)?;
        tracing::info!(target: "stages", "l2 container started on port: {}", l2_port);

        Ok(())
    }
}
