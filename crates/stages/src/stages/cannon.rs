use eyre::Result;
use op_primitives::Monorepo;
use std::process::Command;
use std::rc::Rc;

/// Cannon Prestate Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Prestate {
    monorepo: Rc<Monorepo>,
}

impl crate::Stage for Prestate {
    /// Executes the cannon prestate stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing cannon prestate stage");

        let monorepo = self.monorepo.path();
        let l2_genesis_file = self.monorepo.l2_genesis();

        if l2_genesis_file.exists() {
            tracing::info!(target: "stages", "l2 genesis file already found");
            return Ok(());
        }

        let op_program_bin = monorepo.join("op-program/bin");
        if std::fs::metadata(op_program_bin).is_ok() {
            tracing::info!(target: "stages", "cannon prestate already generated");
            return Ok(());
        }

        let make = Command::new("make")
            .args(["cannon-prestate"])
            .current_dir(monorepo)
            .output()?;

        if !make.status.success() {
            eyre::bail!(
                "failed to generate cannon prestate: {}",
                String::from_utf8_lossy(&make.stderr)
            );
        }

        Ok(())
    }
}

impl Prestate {
    /// Creates a new stage.
    pub fn new(monorepo: Rc<Monorepo>) -> Self {
        Self { monorepo }
    }
}
