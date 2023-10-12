use eyre::Result;
use op_primitives::{Artifacts, Monorepo};
use std::path::Path;
use std::process::Command;
use std::rc::Rc;

/// Devnet Allocs Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Allocs {
    artifacts: Rc<Artifacts>,
    monorepo: Rc<Monorepo>,
}

impl crate::Stage for Allocs {
    /// Executes the allocs stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing allocs stage");

        let l2_genesis_file = self.monorepo.l2_genesis();
        if l2_genesis_file.exists() {
            tracing::info!(target: "stages", "l2 genesis file already found");
            return Ok(());
        }

        let allocs = Command::new("make")
            .args(["devnet-allocs"])
            .current_dir(self.monorepo.path())
            .output()?;
        if !allocs.status.success() {
            eyre::bail!(
                "failed to generate devnet allocs: {}",
                String::from_utf8_lossy(&allocs.stderr)
            );
        }

        self.artifacts
            .copy_from(Path::new(".devnet/addresses.json"))?;
        self.artifacts
            .copy_from(Path::new(".devnet/allocs-l1.json"))?;

        Ok(())
    }
}

impl Allocs {
    /// Creates a new stage.
    pub fn new(artifacts: Rc<Artifacts>, monorepo: Rc<Monorepo>) -> Self {
        Self {
            artifacts,
            monorepo,
        }
    }
}
