use async_trait::async_trait;
use eyre::Result;
use op_primitives::Monorepo;
use std::process::Command;
use std::sync::Arc;

/// Fault proof Prestate Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Prestate {
    monorepo: Arc<Monorepo>,
}

#[async_trait]
impl crate::Stage for Prestate {
    /// Executes the fault proof prestate stage.
    async fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing fault proof prestate stage");

        let monorepo = self.monorepo.path();
        let l2_genesis_file = self.monorepo.l2_genesis();

        if l2_genesis_file.exists() {
            tracing::info!(target: "stages", "l2 genesis file already found");
            return Ok(());
        }

        let op_program_bin = monorepo.join("op-program/bin");
        if std::fs::metadata(op_program_bin).is_ok() {
            tracing::info!(target: "stages", "Fault proof prestate already generated");
            return Ok(());
        }

        let make = Command::new("make")
            .args(["cannon-prestate"])
            .current_dir(monorepo)
            .output()?;

        if !make.status.success() {
            eyre::bail!(
                "failed to generate fault proof prestate: {}",
                String::from_utf8_lossy(&make.stderr)
            );
        }

        Ok(())
    }
}

impl Prestate {
    /// Creates a new stage.
    pub fn new(monorepo: Arc<Monorepo>) -> Self {
        Self { monorepo }
    }
}
