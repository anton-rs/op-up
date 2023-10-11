use std::path::PathBuf;

use eyre::Result;

/// Deploy Config Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DeployConfig {
    /// The deploy config file.
    pub config: PathBuf,
    /// The genesis timestamp.
    pub genesis_timestamp: u64,
}

impl crate::Stage for DeployConfig {
    /// Executes the [DeployConfig] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing deploy config stage");
        let mut deploy_config = crate::json::read_json(&self.config)?;
        let hex_timestamp = format!("{:#x}", self.genesis_timestamp);
        crate::json::set_json_property(
            &mut deploy_config,
            "l1GenesisBlockTimestamp",
            hex_timestamp,
        );
        crate::json::set_json_property(&mut deploy_config, "l1StartingBlockTag", "earliest");
        crate::json::write_json(&self.config, &deploy_config)?;
        Ok(())
    }
}

impl DeployConfig {
    /// Creates a new stage.
    pub fn new(config: Option<PathBuf>, timestamp: u64) -> Self {
        Self {
            config: config.unwrap_or(Self::get_deploy_config_file_unsafe()),
            genesis_timestamp: timestamp,
        }
    }

    /// Returns a [PathBuf] for the deploy config file.
    ///
    /// # Panics
    ///
    /// Panics if the [project_root::get_project_root] function call fails to return a valid
    /// project root [PathBuf].
    pub fn get_deploy_config_file_unsafe() -> PathBuf {
        let proj_root = project_root::get_project_root().expect("Failed to get project root");
        let op_up_dir = proj_root.as_path();
        op_up_dir
            .join("optimism")
            .join("packages/contracts-bedrock")
            .join("deploy-config")
            .join("devnetL1.json")
    }
}
