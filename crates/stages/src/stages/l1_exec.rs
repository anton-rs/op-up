use eyre::Result;
use op_composer::CreateImageOptions;
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

        let image_name = "ethereum/client-go:v1.12.2".to_string();
        let working_dir = project_root::get_project_root()?.join("docker");
        let l1_genesis = self.artifacts.l1_genesis();
        let jwt_secret = self.artifacts.jwt_secret();

        let image_config = CreateImageOptions {
            from_image: image_name.as_str(),
            ..Default::default()
        };
        self.l1_exec.create_image(image_config).await?;

        let config = ContainerConfig {
            cmd: Some(vec![
                "/bin/sh".to_string(), // TODO: fix this: we need to place here the actual geth start command
                "geth-entrypoint.sh".to_string(),
            ]),
            image: Some(image_name),
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

        let container_id = self
            .l1_exec
            .create_container(&self.l1_client, config)
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

// For reference, here is the geth startup command in geth-entrypoint.sh:
//
// exec geth \
// 	--datadir="$GETH_DATA_DIR" \
// 	--verbosity="$VERBOSITY" \
// 	--http \
// 	--http.corsdomain="*" \
// 	--http.vhosts="*" \
// 	--http.addr=0.0.0.0 \
// 	--http.port="$RPC_PORT" \
// 	--http.api=web3,debug,eth,txpool,net,engine \
// 	--ws \
// 	--ws.addr=0.0.0.0 \
// 	--ws.port="$WS_PORT" \
// 	--ws.origins="*" \
// 	--ws.api=debug,eth,txpool,net,engine \
// 	--syncmode=full \
// 	--nodiscover \
// 	--maxpeers=1 \
// 	--networkid=$CHAIN_ID \
// 	--unlock=$BLOCK_SIGNER_ADDRESS \
// 	--mine \
// 	--miner.etherbase=$BLOCK_SIGNER_ADDRESS \
// 	--password="$GETH_DATA_DIR"/password \
// 	--allow-insecure-unlock \
// 	--rpc.allow-unprotected-txs \
// 	--authrpc.addr="0.0.0.0" \
// 	--authrpc.port="8551" \
// 	--authrpc.vhosts="*" \
// 	--authrpc.jwtsecret=/config/test-jwt-secret.txt \
// 	--gcmode=archive \
// 	--metrics \
// 	--metrics.addr=0.0.0.0 \
// 	--metrics.port=6060 \
// 	"$@"
