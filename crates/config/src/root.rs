use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A helper wrapper around the root path used during Config detection
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
pub struct RootPath(pub PathBuf);

impl Default for RootPath {
    fn default() -> Self {
        ".".into()
    }
}

impl<P: Into<PathBuf>> From<P> for RootPath {
    fn from(p: P) -> Self {
        RootPath(p.into())
    }
}

impl AsRef<Path> for RootPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
