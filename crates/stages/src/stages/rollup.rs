use async_trait::async_trait;
use eyre::Result;
use maplit::hashmap;
use op_composer::{
    bind_host_port, BuildContext, Composer, Config, CreateVolumeOptions, HostConfig,
};
use op_primitives::{Artifacts, Monorepo, RollupClient};
use std::sync::Arc;

/// Rollup Stage
#[derive(Debug)]
pub struct Rollup {
    rollup_port: Option<u16>,
    rollup_client: RollupClient,
    rollup_exec: Arc<Composer>,
    monorepo: Arc<Monorepo>,
    artifacts: Arc<Artifacts>,
}

const CONTAINER_NAME: &str = "opup-rollup";

#[async_trait]
impl crate::Stage for Rollup {
    /// Executes the [Rollup] stage.
    async fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing rollup stage");

        match self.rollup_client {
            RollupClient::OpNode => self.start_op_node().await,
            _ => unimplemented!("rollup client not implemented: {}", self.rollup_client),
        }
    }
}

impl Rollup {
    /// Creates a new stage.
    pub fn new(
        rollup_port: Option<u16>,
        rollup_client: RollupClient,
        rollup_exec: Arc<Composer>,
        monorepo: Arc<Monorepo>,
        artifacts: Arc<Artifacts>,
    ) -> Self {
        Self {
            rollup_port,
            rollup_client,
            rollup_exec,
            monorepo,
            artifacts,
        }
    }

    /// Starts Op-Node in a Docker container.
    pub async fn start_op_node(&self) -> Result<()> {
        let image_name = "opup-op-node".to_string();
        let working_dir = project_root::get_project_root()?.join("docker");
        let monorepo = self.monorepo.path();
        let rollup_genesis = self.artifacts.rollup_genesis();
        let rollup_genesis = rollup_genesis.to_string_lossy();
        let jwt_secret = self.artifacts.jwt_secret();
        let jwt_secret = jwt_secret.to_string_lossy();
        let p2p_node_key = self.artifacts.p2p_node_key();
        let p2p_node_key = p2p_node_key.to_string_lossy();

        let dockerfile = r#"
            ARG BUILDPLATFORM
            FROM --platform=$BUILDPLATFORM golang:1.21.1-alpine3.18 as builder
            ARG VERSION=v0.0.0
            RUN apk add --no-cache make gcc musl-dev linux-headers git jq bash
            COPY ./go.mod /app/go.mod
            COPY ./go.sum /app/go.sum
            WORKDIR /app
            RUN go mod download
            # build op-node with the shared go.mod & go.sum files
            COPY ./op-node /app/op-node
            COPY ./op-chain-ops /app/op-chain-ops
            COPY ./op-service /app/op-service
            COPY ./op-bindings /app/op-bindings
            WORKDIR /app/op-node
            ARG TARGETOS TARGETARCH
            RUN make op-node VERSION="$VERSION" GOOS=$TARGETOS GOARCH=$TARGETARCH
            FROM alpine:3.18
            COPY --from=builder /app/op-node/bin/op-node /usr/local/bin
            COPY op-node-entrypoint.sh /op-node-entrypoint.sh
            ENTRYPOINT ["/bin/sh", "/op-node-entrypoint.sh"]
        "#;

        let context = BuildContext::from_dockerfile(dockerfile)
            .add_build_arg("BUILDPLATFORM", "linux/arm64") // TODO: this should be dynamic
            .add_build_arg("TARGETOS", "linux")
            .add_build_arg("TARGETARCH", "arm64")
            .add_file(monorepo.join("go.mod"), "go.mod")
            .add_file(monorepo.join("go.sum"), "go.sum")
            .add_dir(monorepo.join("op-node"), "op-node")
            .add_dir(monorepo.join("op-chain-ops"), "op-chain-ops")
            .add_dir(monorepo.join("op-service"), "op-service")
            .add_dir(monorepo.join("op-bindings"), "op-bindings")
            .add_file(
                working_dir.join("op-node-entrypoint.sh"),
                "op-node-entrypoint.sh",
            );
        self.rollup_exec.build_image(&image_name, context).await?;

        let op_log_volume = CreateVolumeOptions {
            name: "op_log",
            driver: "local",
            ..Default::default()
        };
        self.rollup_exec.create_volume(op_log_volume).await?;

        let config = Config {
            image: Some(image_name),
            working_dir: Some(working_dir.to_string_lossy().to_string()),
            exposed_ports: Some(hashmap! {
                "8545".to_string() => hashmap!{},
                "6060".to_string() => hashmap!{},
                "9003".to_string() => hashmap!{},
                "7300".to_string() => hashmap!{},
            }),
            host_config: Some(HostConfig {
                port_bindings: Some(hashmap! {
                    "8545".to_string() => bind_host_port(7545),
                    "6060".to_string() => bind_host_port(6060),
                    "9003".to_string() => bind_host_port(9003),
                    "7300".to_string() => bind_host_port(7300),
                }),
                binds: Some(vec![
                    "op_log:/op_log".to_string(),
                    format!("{}:/rollup.json", rollup_genesis),
                    format!("{}:/config/test-jwt-secret.txt", jwt_secret),
                    format!("{}:/config/p2p-node-key.txt", p2p_node_key),
                ]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let container_id = self
            .rollup_exec
            .create_container(CONTAINER_NAME, config, true)
            .await?
            .id;
        tracing::info!(target: "stages", "rollup container created: {}", container_id);

        self.rollup_exec.start_container(&container_id).await?;

        let rollup_port = self.rollup_port.unwrap_or(op_config::ROLLUP_PORT);
        crate::net::wait_up(rollup_port, 30, 1)?;
        tracing::info!(target: "stages", "rollup container started on port: {}", rollup_port);

        Ok(())
    }
}
