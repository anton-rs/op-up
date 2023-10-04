use serde::Serialize;

pub mod config;
use config::StageConfig;

/// StageProvider
///
/// The stage provider is an abstract trait that can be implemented by any
/// downstream stage.
#[typetag::serde(tag = "type")]
pub trait StageProvider: std::fmt::Debug + Send + Sync {
    /// Returns the stage configuration.
    fn config(&self) -> &StageConfig;
}
