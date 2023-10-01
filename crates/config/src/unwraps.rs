use eyre::Result;
use figment::{
    value::{Dict, Map, Value},
    Error, Metadata, Profile, Provider,
};

/// Unwraps a profile reducing the key depth
///
/// For example given:
///
/// ```toml
/// [wrapping_key.profile]
/// key = "value"
/// ```
///
/// UnwrapProfileProvider will output:
///
/// ```toml
/// [profile]
/// key = "value"
/// ```
pub struct UnwrapProfileProvider<P> {
    provider: P,
    wrapping_key: Profile,
    profile: Profile,
}

impl<P> UnwrapProfileProvider<P> {
    pub fn new(provider: P, wrapping_key: impl Into<Profile>, profile: impl Into<Profile>) -> Self {
        Self {
            provider,
            wrapping_key: wrapping_key.into(),
            profile: profile.into(),
        }
    }
}

impl<P: Provider> Provider for UnwrapProfileProvider<P> {
    fn metadata(&self) -> Metadata {
        self.provider.metadata()
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        self.provider.data().and_then(|mut data| {
            if let Some(profiles) = data.remove(&self.wrapping_key) {
                for (profile_str, profile_val) in profiles {
                    let profile = Profile::new(&profile_str);
                    if profile != self.profile {
                        continue;
                    }
                    match profile_val {
                        Value::Dict(_, dict) => return Ok(profile.collect(dict)),
                        bad_val => {
                            let mut err = Error::from(figment::error::Kind::InvalidType(
                                bad_val.to_actual(),
                                "dict".into(),
                            ));
                            err.metadata = Some(self.provider.metadata());
                            err.profile = Some(self.profile.clone());
                            return Err(err);
                        }
                    }
                }
            }
            Ok(Default::default())
        })
    }

    fn profile(&self) -> Option<Profile> {
        Some(self.profile.clone())
    }
}
