use eyre::Result;
use figment::{
    providers::{Env, Serialized},
    value::{Dict, Map, Value},
    Figment, Metadata, Profile, Provider,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::PathBuf;
use tracing::trace;

use strum::IntoEnumIterator;

use op_primitives::{ChallengerAgent, L1Client, L2Client, RollupClient};

use crate::error::{ExtractConfigError, OpStackConfigError};
use crate::optional::OptionalStrictProfileProvider;
use crate::rename::RenameProfileProvider;
use crate::root::RootPath;
use crate::toml::TomlFileProvider;
use crate::unwraps::UnwrapProfileProvider;
use crate::wraps::WrapProfileProvider;

/// OP Stack Stage Configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StageConfig {
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
