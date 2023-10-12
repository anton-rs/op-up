use eyre::Result;
use op_primitives::{path_to_str, Monorepo};
use std::process::Command;
use std::rc::Rc;

/// L1 Genesis Stage
#[derive(Debug, Default, Clone, PartialEq)]
pub struct L1Genesis {
    monorepo: Rc<Monorepo>,
    genesis_timestamp: u64,
}

impl crate::Stage for L1Genesis {
    /// Executes the [L1Genesis] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing l1 genesis stage");

        let deploy_config = self.monorepo.deploy_config();
        let deploy_config = path_to_str!(deploy_config)?;
        let allocs = self.monorepo.allocs();
        let allocs = path_to_str!(allocs)?;
        let l1_genesis = self.monorepo.l1_genesis();
        let addresses_json = self.monorepo.addresses_json();
        let addresses_json = path_to_str!(addresses_json)?;
        let op_node_dir = self.monorepo.op_node_dir();

        if l1_genesis.exists() {
            tracing::info!(target: "stages", "L1 genesis already found.");
            return Ok(());
        }

        tracing::info!(target: "stages", "Creating L1 genesis...");
        let genesis_template =
            op_primitives::genesis::genesis_template_string(self.genesis_timestamp)
                .ok_or_else(|| eyre::eyre!("Could not create genesis template"))?;
        std::fs::write(&l1_genesis, genesis_template)?;
        let l1_genesis_str = path_to_str!(l1_genesis)?;
        let l1_genesis = Command::new("go")
            .args(["run", "cmd/main.go", "genesis", "l1"])
            .args(["--deploy-config", deploy_config])
            .args(["--l1-allocs", allocs])
            .args(["--l1-deployments", addresses_json])
            .args(["--outfile.l1", l1_genesis_str])
            .current_dir(op_node_dir)
            .output()?;

        if !l1_genesis.status.success() {
            eyre::bail!(
                "failed to create l1 genesis: {}",
                String::from_utf8_lossy(&l1_genesis.stderr)
            );
        }

        Ok(())
    }
}

impl L1Genesis {
    /// Creates a new stage.
    pub fn new(monorepo: Rc<Monorepo>, genesis_timestamp: u64) -> Self {
        Self {
            monorepo,
            genesis_timestamp,
        }
    }
}
