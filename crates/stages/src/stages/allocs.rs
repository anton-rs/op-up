use std::process::Command;
use std::rc::Rc;

use eyre::Result;

use op_primitives::Monorepo;

/// Devnet Allocs Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Allocs {
    /// The optimism monorepo.
    pub monorepo: Rc<Monorepo>,
}

impl crate::Stage for Allocs {
    /// Executes the allocs stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing allocs stage");

        let l2_genesis_file = self.monorepo.l2_genesis_file();
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

        let copy_addr = Command::new("cp")
            .args([".devnet/addresses.json", "../.devnet/"])
            .current_dir(self.monorepo.path())
            .output()?;
        if !copy_addr.status.success() {
            eyre::bail!(
                "failed to copy l1 deployments: {}",
                String::from_utf8_lossy(&copy_addr.stderr)
            );
        }

        let copy_allocs = Command::new("cp")
            .args([".devnet/allocs-l1.json", "../.devnet/"])
            .current_dir(self.monorepo.path())
            .output()?;
        if !copy_allocs.status.success() {
            eyre::bail!(
                "failed to copy allocs: {}",
                String::from_utf8_lossy(&copy_allocs.stderr)
            );
        }

        Ok(())
    }
}

impl Allocs {
    /// Creates a new stage.
    pub fn new(monorepo: Rc<Monorepo>) -> Self {
        Self { monorepo }
    }
}
