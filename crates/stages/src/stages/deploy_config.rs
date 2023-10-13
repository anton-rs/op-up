use async_trait::async_trait;
use eyre::Result;
use op_primitives::Monorepo;
use std::sync::Arc;

/// Deploy Config Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DeployConfig {
    monorepo: Arc<Monorepo>,
    genesis_timestamp: u64,
}

#[async_trait]
impl crate::Stage for DeployConfig {
    /// Executes the [DeployConfig] stage.
    async fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing deploy config stage");
        let deploy_config_file = self.monorepo.deploy_config();
        let mut deploy_config = crate::json::read_json(&deploy_config_file)?;
        let hex_timestamp = format!("{:#x}", self.genesis_timestamp);
        crate::json::set_json_property(
            &mut deploy_config,
            "l1GenesisBlockTimestamp",
            hex_timestamp,
        );
        crate::json::set_json_property(&mut deploy_config, "l1StartingBlockTag", "earliest");
        crate::json::write_json(&deploy_config_file, &deploy_config)?;
        Ok(())
    }
}

impl DeployConfig {
    /// Creates a new stage.
    pub fn new(monorepo: Arc<Monorepo>, genesis_timestamp: u64) -> Self {
        Self {
            monorepo,
            genesis_timestamp,
        }
    }
}
