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
pub struct RenameProfileProvider<P> {
    provider: P,
    from: Profile,
    to: Profile,
}

impl<P> RenameProfileProvider<P> {
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
