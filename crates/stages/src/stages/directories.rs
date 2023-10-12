use eyre::Result;
use std::rc::Rc;

use op_primitives::{Artifacts, Monorepo};

/// Directories Stage
///
/// The directories stage handles the cloning of git repositories and
/// other directories construction required for subsequent stages. This
/// stage should be executed early in the sequential [Stages] execution
/// pipeline.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Directories {
    artifacts: Rc<Artifacts>,
    monorepo: Rc<Monorepo>,
}

impl crate::Stage for Directories {
    /// Executes the [Directories] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing directories stage");
        self.artifacts.create()?;

        self.monorepo.obtain_from_source()
    }
}

impl Directories {
    /// Creates a new stage.
    pub fn new(artifacts: Rc<Artifacts>, monorepo: Rc<Monorepo>) -> Self {
        Self {
            artifacts,
            monorepo,
        }
    }
}
