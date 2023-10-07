#![allow(unconditional_recursion)]
use serde::{ser::SerializeStruct, Deserialize, Serialize};

/// The stage config.
pub mod config;
use config::StageConfig;

/// StageProvider
#[derive(Debug, Clone)]
pub struct StageProvider<'a> {
    /// The internal Stage.
    pub inner: Option<&'a dyn Stage>,
}

impl PartialEq for StageProvider<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.inner
            .is_some_and(|i| other.inner.is_some_and(|o| o.name() == i.name()))
    }
}

impl<'a> Serialize for StageProvider<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let state = serializer.serialize_struct("StageProvider", 1)?;
        // state.serialize_field("inner", &None::<Option<_>>)?;
        state.end()
    }
}

impl<'a> Deserialize<'a> for StageProvider<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let stage_provider: StageProvider<'a> = serde::Deserialize::deserialize(deserializer)?;
        Ok(stage_provider)
        // deserializer.deserialize_any()
        // let s = Box::new(Stage::deserialize(deserializer)?);
        // Ok(StageProvider { inner: None })
    }
}

/// Stage
///
/// The stage provider is an abstract trait that can be implemented by any
/// downstream stage.
pub trait Stage: std::fmt::Debug + Send + Sync {
    /// Returns the stage configuration.
    fn config(&self) -> &StageConfig;

    /// Returns the state name.
    fn name(&self) -> &str;
}
