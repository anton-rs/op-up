use eyre::Result;
use figment::{
    providers::{Env, Format, Toml},
    value::{Dict, Map},
    Error, Metadata, Profile, Provider,
};
use std::path::{Path, PathBuf};

/// A convenience provider to retrieve a toml file.
/// This will return an error if the env var is set but the file does not exist
#[derive(Debug)]
pub struct TomlFileProvider {
    /// The env var to read the file from.
    pub env_var: Option<&'static str>,
    /// The default file to read from.
    pub default: PathBuf,
    /// Cached data.
    pub cache: Option<Result<Map<Profile, Dict>, Error>>,
}

impl TomlFileProvider {
    /// Creates a new `TomlFileProvider` from the given env var and default file.
    pub fn new(env_var: Option<&'static str>, default: impl Into<PathBuf>) -> Self {
        Self {
            env_var,
            default: default.into(),
            cache: None,
        }
    }

    /// Returns the env var if set.
    pub fn env_val(&self) -> Option<String> {
        self.env_var.and_then(Env::var)
    }

    /// Returns the file path.
    pub fn file(&self) -> PathBuf {
        self.env_val()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.default.clone())
    }

    /// Returns true if the file is missing.
    pub fn is_missing(&self) -> bool {
        if let Some(file) = self.env_val() {
            let path = Path::new(&file);
            if !path.exists() {
                return true;
            }
        }
        false
    }

    /// Returns the cached data if set, otherwise reads the file.
    pub fn cached(mut self) -> Self {
        self.cache = Some(self.read());
        self
    }

    /// Reads the file.
    pub fn read(&self) -> Result<Map<Profile, Dict>, Error> {
        use serde::de::Error as _;
        if let Some(file) = self.env_val() {
            let path = Path::new(&file);
            if !path.exists() {
                return Err(Error::custom(format!(
                    "Config file `{}` set in env var `{}` does not exist",
                    file,
                    self.env_var.unwrap()
                )));
            }
            Toml::file(file)
        } else {
            Toml::file(&self.default)
        }
        .nested()
        .data()
    }
}

impl Provider for TomlFileProvider {
    fn metadata(&self) -> Metadata {
        if self.is_missing() {
            Metadata::named("TOML file provider")
        } else {
            Toml::file(self.file()).nested().metadata()
        }
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        if let Some(cache) = self.cache.as_ref() {
            cache.clone()
        } else {
            self.read()
        }
    }
}
