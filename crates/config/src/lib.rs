#![doc = include_str!("../README.md")]
#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    rustdoc::all
)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

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

mod error;
pub use error::{ExtractConfigError, OpStackConfigError};

mod toml;
pub use toml::TomlFileProvider;

mod rename;
pub use rename::RenameProfileProvider;

mod wraps;
pub use wraps::WrapProfileProvider;

mod optional;
pub use optional::OptionalStrictProfileProvider;

mod unwraps;
pub use unwraps::UnwrapProfileProvider;

use op_stack::components::{
    challenger::ChallengerAgent, layer_one::L1Client, layer_two::L2Client, rollup::RollupClient,
};

/// RootPath convenience re-export
mod root;
pub use root::RootPath;

/// OP Stack Configuration
///
/// # Defaults
///
/// All configuration values have a default, documented in the [fields](#fields)
/// section below. [`Config::default()`] returns the default values for
/// the default profile while [`Config::with_root()`] returns the values based on the given
/// directory. [`Config::load()`] starts with the default profile and merges various providers into
/// the config, same for [`Config::load_with_root()`], but there the default values are determined
/// by [`Config::with_root()`]
///
/// # Provider Details
///
/// `Config` is a Figment [`Provider`] with the following characteristics:
///
///   * **Profile**
///
///     The profile is set to the value of the `profile` field.
///
///   * **Metadata**
///
///     This provider is named `OP Stack Config`. It does not specify a
///     [`Source`](figment::Source) and uses default interpolation.
///
///   * **Data**
///
///     The data emitted by this provider are the keys and values corresponding
///     to the fields and values of the structure. The dictionary is emitted to
///     the "default" meta-profile.
///
/// Note that these behaviors differ from those of [`Config::figment()`].
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Config {
    /// The selected profile. **(default: _default_ `default`)**
    ///
    /// **Note:** This field is never serialized nor deserialized. When a
    /// `Config` is merged into a `Figment` as a `Provider`, this profile is
    /// selected on the `Figment`. When a `Config` is extracted, this field is
    /// set to the extracting Figment's selected `Profile`.
    #[serde(skip)]
    pub profile: Profile,

    /// The path to the op stack artifact directory. **(default: _default_ `.stack`)**
    pub artifacts: PathBuf,

    /// The type of L1 Client to use. **(default: _default_ `L1Client::Geth`)**
    pub l1_client: L1Client,
    /// The type of L2 Client to use. **(default: _default_ `L2Client::Geth`)**
    pub l2_client: L2Client,
    /// The type of Rollup Client to use. **(default: _default_ `RollupClient::Node`)**
    pub rollup_client: RollupClient,

    /// The challenger agent to use. **(default: _default_ `ChallengerAgent::Node`)**
    pub challenger: ChallengerAgent,

    /// Enable Sequencing. **(default: _default_ `false`)**
    pub enable_sequencing: bool,
    /// Enable Fault Proofs. **(default: _default_ `false`)**
    pub enable_fault_proofs: bool,

    /// JWT secret that should be used for any rpc calls
    pub eth_rpc_jwt: Option<String>,

    /// The root path where the config detection started from, `Config::with_root`
    #[doc(hidden)]
    // Skip serialization here, so it won't be included in the [`Config::to_string()`]
    // representation, but will be deserialized from `Figment` so that commands can
    // override it.
    #[serde(rename = "root", default, skip_serializing)]
    pub __root: RootPath,
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Macro to create a selection prompt.
#[macro_export]
macro_rules! make_selection {
    ($name:ident, $prompt:expr, $options:expr) => {
        let $name = inquire::Select::new($prompt, $options)
            .without_help_message()
            .prompt()?
            .to_string();
    };
}

impl Config {
    /// The default profile: "default"
    pub const DEFAULT_PROFILE: Profile = Profile::const_new("default");

    /// TOML section for profiles
    pub const PROFILE_SECTION: &'static str = "profile";

    /// File name of config toml file
    pub const FILE_NAME: &'static str = "stack.toml";

    /// The name of the directory rollup reserves for itself under the user's home directory: `~`
    pub const STACK_DIR_NAME: &'static str = ".stack";

    /// Standalone sections in the config which get integrated into the selected profile
    pub const STANDALONE_SECTIONS: &'static [&'static str] = &[];

    /// Returns the current `Config`
    ///
    /// See `Config::figment`
    #[track_caller]
    pub fn load() -> Self {
        Config::from_provider(Config::figment())
    }

    /// Returns the current `Config`
    ///
    /// See `Config::figment_with_root`
    #[track_caller]
    pub fn load_with_root(root: impl Into<PathBuf>) -> Self {
        Config::from_provider(Config::figment_with_root(root))
    }

    /// Extract a `Config` from `provider`, panicking if extraction fails.
    ///
    /// # Panics
    ///
    /// If extraction fails, prints an error message indicating the failure and
    /// panics. For a version that doesn't panic, use [`Config::try_from()`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use op_config::Config;
    /// use figment::providers::{Toml, Format, Env};
    ///
    /// // Use default `Figment`, but allow values from `other.toml`
    /// // to supersede its values.
    /// let figment = Config::figment()
    ///     .merge(Toml::file("other.toml").nested());
    ///
    /// let config = Config::from_provider(figment);
    /// ```
    #[track_caller]
    pub fn from_provider<T: Provider>(provider: T) -> Self {
        trace!("load config with provider: {:?}", provider.metadata());
        Self::try_from(provider).unwrap_or_else(|err| panic!("{}", err))
    }

    /// Attempts to extract a `Config` from `provider`, returning the result.
    ///
    /// # Example
    ///
    /// ```rust
    /// use op_config::Config;
    /// use figment::providers::{Toml, Format, Env};
    ///
    /// // Use default `Figment`, but allow values from `other.toml`
    /// // to supersede its values.
    /// let figment = Config::figment()
    ///     .merge(Toml::file("other.toml").nested());
    ///
    /// let config = Config::try_from(figment);
    /// ```
    pub fn try_from<T: Provider>(provider: T) -> Result<Self, ExtractConfigError> {
        let figment = Figment::from(provider);
        let mut config = figment.extract::<Self>().map_err(ExtractConfigError::new)?;
        config.profile = figment.profile().clone();
        Ok(config)
    }

    /// Returns the default figment
    ///
    /// The default figment reads from the following sources, in ascending
    /// priority order:
    ///
    ///   1. [`Config::default()`] (see [defaults](#defaults))
    ///   2. `rollup.toml` _or_ filename in `OP_STACK_CONFIG` environment variable
    ///   3. `OP_STACK_` prefixed environment variables
    ///
    /// The profile selected is the value set in the `OP_STACK_PROFILE`
    /// environment variable. If it is not set, it defaults to `default`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use op_config::Config;
    /// use serde::Deserialize;
    ///
    /// let my_config = Config::figment().extract::<Config>();
    /// ```
    pub fn figment() -> Figment {
        Config::default().into()
    }

    /// Returns the default figment enhanced with additional context extracted from the provided
    /// root, like remappings and directories.
    ///
    /// # Example
    ///
    /// ```rust
    /// use op_config::Config;
    /// use serde::Deserialize;
    ///
    /// let my_config = Config::figment_with_root(".").extract::<Config>();
    /// ```
    pub fn figment_with_root(root: impl Into<PathBuf>) -> Figment {
        Self::with_root(root).into()
    }

    /// Creates a new Config that adds additional context extracted from the provided root.
    ///
    /// # Example
    ///
    /// ```rust
    /// use op_config::Config;
    /// let my_config = Config::with_root(".");
    /// ```
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Config {
            __root: root::RootPath(root.into()),
            ..Config::default()
        }
    }

    /// Creates the artifacts directory if it doesn't exist.
    pub fn create_artifacts_dir(&self) -> Result<()> {
        if !self.artifacts.exists() {
            std::fs::create_dir_all(&self.artifacts)?;
        }
        Ok(())
    }

    /// Returns the selected profile
    ///
    /// If the `STACK_PROFILE` env variable is not set, this returns the `DEFAULT_PROFILE`
    pub fn selected_profile() -> Profile {
        Profile::from_env_or("STACK_PROFILE", Config::DEFAULT_PROFILE)
    }

    /// Returns the path to the global toml file that's stored at `~/.stack/stack.toml`
    pub fn stack_dir_toml() -> Option<PathBuf> {
        Self::stack_dir().map(|p| p.join(Config::FILE_NAME))
    }

    /// Returns the path to the config dir `~/.stack/`
    pub fn stack_dir() -> Option<PathBuf> {
        dirs_next::home_dir().map(|p| p.join(Config::STACK_DIR_NAME))
    }

    /// Sets the l1 client to use via a cli prompt.
    pub fn set_l1_client(&mut self) -> Result<()> {
        make_selection!(
            l1_client,
            "Which L1 execution client would you like to use?",
            L1Client::iter().collect::<Vec<_>>()
        );
        self.l1_client = l1_client.parse()?;
        tracing::debug!(target: "stack", "Nice l1 client choice! You've got great taste ✨");
        Ok(())
    }

    /// Sets the l2 client to use via a cli prompt.
    pub fn set_l2_client(&mut self) -> Result<()> {
        make_selection!(
            l2_client,
            "Which L2 execution client would you like to use?",
            L2Client::iter().collect::<Vec<_>>()
        );
        self.l2_client = l2_client.parse()?;
        tracing::debug!(target: "stack", "Nice l2 client choice! You've got great taste ✨");
        Ok(())
    }

    /// Sets the rollup client to use via a cli prompt.
    pub fn set_rollup_client(&mut self) -> Result<()> {
        make_selection!(
            rollup_client,
            "Which rollup client would you like to use?",
            RollupClient::iter().collect::<Vec<_>>()
        );
        self.rollup_client = rollup_client.parse()?;
        tracing::debug!(target: "stack", "Nice rollup choice! You've got great taste ✨");
        Ok(())
    }

    /// Sets the challenger agent to use via a cli prompt.
    pub fn set_challenger(&mut self) -> Result<()> {
        make_selection!(
            challenger,
            "Which challenger agent would you like to use?",
            ChallengerAgent::iter().collect::<Vec<_>>()
        );
        self.challenger = challenger.parse()?;
        tracing::debug!(target: "stack", "Nice challenger choice! You've got great taste ✨");
        Ok(())
    }

    fn merge_toml_provider(
        mut figment: Figment,
        toml_provider: impl Provider,
        profile: Profile,
    ) -> Figment {
        figment = figment.select(profile.clone());

        // use [profile.<profile>] as [<profile>]
        let mut profiles = vec![Config::DEFAULT_PROFILE];
        if profile != Config::DEFAULT_PROFILE {
            profiles.push(profile.clone());
        }
        let provider = toml_provider; // toml_provider.strict_select(profiles);

        // merge the default profile as a base
        if profile != Config::DEFAULT_PROFILE {
            figment = figment.merge(provider.rename(Config::DEFAULT_PROFILE, profile.clone()));
        }

        // merge the profile
        figment = figment.merge(provider);
        figment
    }
}

impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("OP Stack Config")
    }

    #[track_caller]
    fn data(&self) -> Result<Map<Profile, Dict>, figment::Error> {
        let mut data = Serialized::defaults(self).data()?;
        if let Some(entry) = data.get_mut(&self.profile) {
            entry.insert("root".to_string(), Value::serialize(self.__root.clone())?);
        }
        Ok(data)
    }

    fn profile(&self) -> Option<Profile> {
        Some(self.profile.clone())
    }
}

impl From<Config> for Figment {
    fn from(c: Config) -> Figment {
        let profile = Config::selected_profile();
        let mut figment = Figment::default();

        // merge global toml file
        if let Some(global_toml) = Config::stack_dir_toml().filter(|p| p.exists()) {
            figment = Config::merge_toml_provider(
                figment,
                TomlFileProvider::new(None, global_toml).cached(),
                profile.clone(),
            );
        }
        // merge local toml file
        figment = Config::merge_toml_provider(
            figment,
            TomlFileProvider::new(Some("OP_STACK_CONFIG"), c.__root.0.join(Config::FILE_NAME))
                .cached(),
            profile.clone(),
        );

        // merge environment variables
        figment = figment
            .merge(
                Env::prefixed("OP_STACK_")
                    .ignore(&[
                        "PROFILE",
                        "L1_CLIENT",
                        "L2_CLIENT",
                        "ROLLUP_CLIENT",
                        "CHALLENGER",
                    ])
                    .map(|key| {
                        let key = key.as_str();
                        if Config::STANDALONE_SECTIONS.iter().any(|section| {
                            key.starts_with(&format!("{}_", section.to_ascii_uppercase()))
                        }) {
                            key.replacen('_', ".", 1).into()
                        } else {
                            key.into()
                        }
                    })
                    .global(),
            )
            .select(profile.clone());

        Figment::from(c).merge(figment).select(profile)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            profile: Self::DEFAULT_PROFILE,
            artifacts: PathBuf::from(Self::STACK_DIR_NAME),
            l1_client: L1Client::default(),
            l2_client: L2Client::default(),
            rollup_client: RollupClient::default(),
            challenger: ChallengerAgent::default(),
            enable_sequencing: false,
            enable_fault_proofs: false,
            eth_rpc_jwt: None,
            __root: RootPath::default(),
        }
    }
}

trait ProviderExt: Provider {
    fn rename(
        &self,
        from: impl Into<Profile>,
        to: impl Into<Profile>,
    ) -> RenameProfileProvider<&Self> {
        RenameProfileProvider::new(self, from, to)
    }

    fn wrap(
        &self,
        wrapping_key: impl Into<Profile>,
        profile: impl Into<Profile>,
    ) -> WrapProfileProvider<&Self> {
        WrapProfileProvider::new(self, wrapping_key, profile)
    }

    fn strict_select(
        &self,
        profiles: impl IntoIterator<Item = impl Into<Profile>>,
    ) -> OptionalStrictProfileProvider<&Self> {
        OptionalStrictProfileProvider::new(self, profiles)
    }
}
impl<P: Provider> ProviderExt for P {}
