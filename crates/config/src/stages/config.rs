use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::root::RootPath;

/// OP Stack Stage Configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StageConfig {
    /// The root path of the stage configuration.
    // Skip serialization here, so it won't be included in the [`Config::to_string()`]
    // representation, but will be deserialized from `Figment` so that commands can
    // override it.
    #[serde(rename = "root", default, skip_serializing)]
    pub __root: RootPath,
}

impl Display for StageConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
