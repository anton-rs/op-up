use std::path::PathBuf;

use eyre::Result;

/// Artifacts Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Artifacts {
    /// The artifacts directory.
    pub artifacts: PathBuf,
}

impl crate::Stage for Artifacts {
    /// Executes the [Artifacts] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing artifacts stage");

        if !self.artifacts.exists() {
            tracing::info!(target: "stages", "Creating artifacts directory: {:?}", self.artifacts);
            std::fs::create_dir_all(&self.artifacts)?;
        }

        Ok(())
    }
}

impl Artifacts {
    /// Creates a new stage.
    pub fn new(artifacts: PathBuf) -> Self {
        Self { artifacts }
    }
}
