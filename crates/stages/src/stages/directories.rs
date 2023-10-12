use eyre::Result;

/// Directories Stage
///
/// The directories stage handles the cloning of git repositories and
/// other directories construction required for subsequent stages. This
/// stage should be executed early in the sequential [Stages] execution
/// pipeline.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Directories {
    /// The optimism monorepo.
    pub monorepo: op_primitives::Monorepo,
}

impl crate::Stage for Directories {
    /// Executes the [Directories] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing directories stage");
        self.monorepo.git_clone()
    }
}

impl Directories {
    /// Creates a new stage.
    pub fn new(monorepo: op_primitives::Monorepo) -> Self {
        Self { monorepo }
    }
}
