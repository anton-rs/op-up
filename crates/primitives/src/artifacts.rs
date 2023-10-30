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

    /// Returns the L1 deployments json file (addresses.json) path for the file in the artifacts
    /// directory.
    pub fn l1_deployments(&self) -> PathBuf {
        self.path().join("addresses.json")
    }

    /// Returns the l1 genesis file path.
    pub fn l1_genesis(&self) -> PathBuf {
        self.path().join("genesis-l1.json")
    }

    /// Returns the l2 genesis fle path.
    pub fn l2_genesis(&self) -> PathBuf {
        self.path().join("genesis-l2.json")
    }

    /// Returns the genesis rollup file path.
    pub fn rollup_genesis(&self) -> PathBuf {
        self.path().join("genesis-rollup.json")
    }

    /// Returns the jwt secret file path.
    pub fn jwt_secret(&self) -> PathBuf {
        self.path().join("jwt-secret.txt")
    }

    pub fn p2p_node_key(&self) -> PathBuf {
        self.path().join("p2p-node-key.txt")
    }

    /// Create the artifacts directory if it does not exist.
    pub fn create(&self) -> Result<()> {
        if !self.pwd.exists() {
            tracing::info!(target: "stages", "Creating artifacts directory: {:?}", self.pwd);
            std::fs::create_dir_all(&self.pwd)?;
        }
        Ok(())
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
