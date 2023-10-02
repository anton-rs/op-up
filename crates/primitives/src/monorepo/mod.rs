//! Utilities for working with the Optimism Monorepo.
use std::path::PathBuf;

// todo: this is a temporary holding file for the optimism monorepo helper struct
//       the helper struct should allow for easy interfacing with the monorepo
//       and its various components.

/// The monorepo directory.
#[allow(dead_code)]
pub const MONOREPO_DIR: &str = "optimism";

/// The Optimism Monorepo.
#[derive(Debug, Clone)]
pub struct Monorepo {
    /// The monorepo directory.
    pub dir: PathBuf,
}

impl Monorepo {
    /// Create a new Optimism Monorepo.
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// Clone the Optimism Monorepo.
    pub fn clone() -> eyre::Result<Self> {
        // todo::::
        Ok(Monorepo::new(PathBuf::from(MONOREPO_DIR)))
    }
}
