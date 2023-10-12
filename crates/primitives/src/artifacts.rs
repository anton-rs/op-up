use eyre::Result;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Artifacts
///
/// The artifacts object exposes methods to interact with the artifacts directory.
/// [Artifacts] directory is configurable by the `artifacts` field on the [op_config::Config]
/// object. The default value is `.stack`.
///
/// [Artifacts] contains both intermediate and final outputs from the [op_stages::Stages] pipeline.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Artifacts {
    pwd: PathBuf,
}

impl Artifacts {
    /// Creates a new [Artifacts] instance.
    ///
    /// # Errors
    ///
    /// If the current working directory cannot be determined, an error will be returned.
    pub fn new(artifacts: impl Into<String>) -> Result<Self> {
        Ok(Self {
            pwd: std::env::current_dir()?.join(artifacts.into()),
        })
    }

    /// Returns the path to the artifacts directory.
    pub fn path(&self) -> &Path {
        self.pwd.as_path()
    }

    /// Copies the contents of a given [Path] into the artifacts directory.
    pub fn copy_from(&self, p: &Path) -> Result<()> {
        let p = p.canonicalize()?;
        let p = crate::path_to_str!(p)?;
        let artifacts = crate::path_to_str!(self.path())?;
        let copy = Command::new("cp")
            .args(["-r", p, artifacts])
            .current_dir(&self.pwd)
            .output()?;
        if !copy.status.success() {
            eyre::bail!(
                "failed to copy from {:?} to {:?}: {}",
                p,
                artifacts,
                String::from_utf8_lossy(&copy.stderr)
            )
        }
        Ok(())
    }
}

impl From<&Path> for Artifacts {
    fn from(p: &Path) -> Self {
        Self {
            pwd: p.to_path_buf(),
        }
    }
}
