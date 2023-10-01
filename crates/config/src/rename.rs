use eyre::Result;
use figment::{
    value::{Dict, Map},
    Error, Metadata, Profile, Provider,
};

/// Renames a profile from `from` to `to
///
/// For example given:
///
/// ```toml
/// [from]
/// key = "value"
/// ```
///
/// RenameProfileProvider will output
///
/// ```toml
/// [to]
/// key = "value"
/// ```
#[derive(Debug)]
pub struct RenameProfileProvider<P> {
    provider: P,
    from: Profile,
    to: Profile,
}

impl<P> RenameProfileProvider<P> {
    /// Creates a new `RenameProfileProvider` from the given provider and profiles
    #[allow(dead_code)]
    pub fn new(provider: P, from: impl Into<Profile>, to: impl Into<Profile>) -> Self {
        Self {
            provider,
            from: from.into(),
            to: to.into(),
        }
    }
}

impl<P: Provider> Provider for RenameProfileProvider<P> {
    fn metadata(&self) -> Metadata {
        self.provider.metadata()
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        let mut data = self.provider.data()?;
        if let Some(data) = data.remove(&self.from) {
            return Ok(Map::from([(self.to.clone(), data)]));
        }
        Ok(Default::default())
    }

    fn profile(&self) -> Option<Profile> {
        Some(self.to.clone())
    }
}
