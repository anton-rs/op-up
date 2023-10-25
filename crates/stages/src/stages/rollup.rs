use async_trait::async_trait;
use eyre::Result;
use op_composer::{Composer, CreateVolumeOptions};
use op_primitives::{Artifacts, RollupClient};
use std::sync::Arc;

/// Rollup Stage
#[derive(Debug)]
pub struct Rollup {
    rollup_port: Option<u16>,
    rollup_client: RollupClient,
    rollup_exec: Arc<Composer>,
    artifacts: Arc<Artifacts>,
}

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
        artifacts: Arc<Artifacts>,
    ) -> Self {
        Self {
            rollup_port,
            rollup_client,
            rollup_exec,
            artifacts,
        }
    }

    /// Starts Op-Node in a Docker container.
    pub async fn start_op_node(&self) -> Result<()> {
        let image_name = "opup-op-node".to_string();
        let working_dir = project_root::get_project_root()?.join("docker");
        // let rollup_genesis = self.artifacts.rollup_genesis();
        // let rollup_genesis = rollup_genesis.to_string_lossy();
        // let jwt_secret = self.artifacts.jwt_secret();
        // let jwt_secret = jwt_secret.to_string_lossy();

        // TODO: Use existing op-node docker image instead of manually building this
        let dockerfile = r#"
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
            CMD ["op-node"]
        "#;

        // let rollup_entrypoint = std::fs::read(working_dir.join("rollup-entrypoint.sh"))?;
        // let build_context_files = [("rollup-entrypoint.sh", rollup_entrypoint.as_slice())];
        // self.rollup_exec
        //     .build_image(&image_name, dockerfile, &build_context_files)
        //     .await?;

        let rollup_data_volume = CreateVolumeOptions {
            name: "rollup_data",
            driver: "local",
            ..Default::default()
        };
        self.rollup_exec.create_volume(rollup_data_volume).await?;

        // TODO: Create container

        // TODO: start container

        let rollup_port = self.rollup_port.unwrap_or(op_config::ROLLUP_PORT);
        crate::net::wait_up(rollup_port, 30, 1)?;
        tracing::info!(target: "stages", "rollup container started on port: {}", rollup_port);

        Ok(())
    }
}
