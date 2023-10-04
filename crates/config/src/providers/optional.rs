use eyre::Result;
use figment::{
    value::{Dict, Map},
    Error, Figment, Metadata, Profile, Provider,
};

use super::unwraps::UnwrapProfileProvider;

/// Extracts the profile from the `profile` key and using the original key as backup, merging
/// values where necessary
///
/// For example given:
///
/// ```toml
/// [profile.cool]
/// key = "value"
///
/// [cool]
/// key2 = "value2"
/// ```
///
/// OptionalStrictProfileProvider will output:
///
/// ```toml
/// [cool]
/// key = "value"
/// key2 = "value2"
/// ```
///
/// And emit a deprecation warning
#[derive(Debug)]
pub struct OptionalStrictProfileProvider<P> {
    provider: P,
    profiles: Vec<Profile>,
}

impl<P> OptionalStrictProfileProvider<P> {
    pub(crate) const PROFILE_PROFILE: Profile = Profile::const_new("profile");

    /// Creates a new `OptionalStrictProfileProvider` from the given provider and profiles
    #[allow(dead_code)]
    pub fn new(provider: P, profiles: impl IntoIterator<Item = impl Into<Profile>>) -> Self {
        Self {
            provider,
            profiles: profiles.into_iter().map(|profile| profile.into()).collect(),
        }
    }
}

impl<P: Provider> Provider for OptionalStrictProfileProvider<P> {
    fn metadata(&self) -> Metadata {
        self.provider.metadata()
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        let mut figment = Figment::from(&self.provider);
        for profile in &self.profiles {
            figment = figment.merge(UnwrapProfileProvider::new(
                &self.provider,
                Self::PROFILE_PROFILE,
                profile.clone(),
            ));
        }
        figment.data().map_err(|err| {
            // figment does tag metadata and tries to map metadata to an error, since we use a new
            // figment in this provider this new figment does not know about the metadata of the
            // provider and can't map the metadata to the error. Therefor we return the root error
            // if this error originated in the provider's data.
            if let Err(root_err) = self.provider.data() {
                return root_err;
            }
            err
        })
    }

    fn profile(&self) -> Option<Profile> {
        self.profiles.last().cloned()
    }
}
