use std::path::PathBuf;

use eyre::Result;

/// Directories Stage
///
/// The directories stage handles the cloning of git repositories and
/// other directories construction required for subsequent stages. This
/// stage should be executed early in the sequential [Stages] execution
/// pipeline.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Directories {
    /// The optimism monorepo directory.
    pub monorepo: PathBuf,
}

impl crate::Stage for Directories {
    /// Executes the [Directories] stage.
    fn execute(&self) -> Result<()> {
        tracing::info!(target: "stages", "Executing directories stage");
        let proj_root = project_root::get_project_root()?;
        op_primitives::Monorepo::clone(proj_root.as_path())?;
        Ok(())
    }
}

impl Directories {
    /// Creates a new stage.
    pub fn new(monorepo: Option<PathBuf>) -> Self {
        Self {
            monorepo: monorepo.unwrap_or(Self::get_op_monorepo_dir_unsafe()),
        }
    }

    /// Returns a [PathBuf] for the monorepo directory.
    ///
    /// # Panics
    ///
    /// Panics if the [project_root::get_project_root] function call fails to return a valid
    /// project root [PathBuf].
    pub fn get_op_monorepo_dir_unsafe() -> PathBuf {
        let proj_root = project_root::get_project_root().expect("Failed to get project root");
        let op_up_dir = proj_root.as_path();
        op_up_dir.join("optimism")
    }
}
