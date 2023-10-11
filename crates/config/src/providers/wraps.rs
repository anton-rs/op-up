use eyre::Result;
use figment::{
    value::{Dict, Map, Value},
    Error, Metadata, Profile, Provider,
};
use inflector::Inflector;

/// Wraps a profile in another profile
///
/// For example given:
///
/// ```toml
/// [profile]
/// key = "value"
/// ```
///
/// WrapProfileProvider will output:
///
/// ```toml
/// [wrapping_key.profile]
/// key = "value"
/// ```
#[derive(Debug)]
pub struct WrapProfileProvider<P> {
    provider: P,
    wrapping_key: Profile,
    profile: Profile,
}

impl<P> WrapProfileProvider<P> {
    /// Creates a new `WrapProfileProvider` from the given provider and profiles.
    #[allow(dead_code)]
    pub fn new(provider: P, wrapping_key: impl Into<Profile>, profile: impl Into<Profile>) -> Self {
        Self {
            provider,
            wrapping_key: wrapping_key.into(),
            profile: profile.into(),
        }
    }
}

impl<P: Provider> Provider for WrapProfileProvider<P> {
    fn metadata(&self) -> Metadata {
        self.provider.metadata()
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        if let Some(inner) = self.provider.data()?.remove(&self.profile) {
            let value = Value::from(inner);
            let dict = [(self.profile.to_string().to_snake_case(), value)]
                .into_iter()
                .collect();
            Ok(self.wrapping_key.collect(dict))
        } else {
            Ok(Default::default())
        }
    }

    fn profile(&self) -> Option<Profile> {
        Some(self.profile.clone())
    }
}
