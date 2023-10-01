//! error handling and solc error codes
use figment::providers::{Format, Toml};
use std::{collections::HashSet, error::Error, fmt};

/// The message shown upon panic if the config could not be extracted from the figment
pub const FAILED_TO_EXTRACT_CONFIG_PANIC_MSG: &str = "failed to extract config:";

/// Represents a failed attempt to extract `Config` from a `Figment`
#[derive(Clone, Debug, PartialEq)]
pub struct ExtractConfigError {
    /// error thrown when extracting the `Config`
    pub(crate) error: figment::Error,
}

impl ExtractConfigError {
    /// Wraps the figment error
    pub fn new(error: figment::Error) -> Self {
        Self { error }
    }
}

impl fmt::Display for ExtractConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut unique_errors = Vec::with_capacity(self.error.count());
        let mut unique = HashSet::with_capacity(self.error.count());
        for err in self.error.clone().into_iter() {
            let err = if err
                .metadata
                .as_ref()
                .map(|meta| meta.name.contains(Toml::NAME))
                .unwrap_or_default()
            {
                OpStackConfigError::Toml(err)
            } else {
                OpStackConfigError::Other(err)
            };

            if unique.insert(err.to_string()) {
                unique_errors.push(err);
            }
        }
        writeln!(f, "{FAILED_TO_EXTRACT_CONFIG_PANIC_MSG}")?;
        for err in unique_errors {
            writeln!(f, "{err}")?;
        }
        Ok(())
    }
}

impl Error for ExtractConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Error::source(&self.error)
    }
}

/// Represents an error that can occur when constructing the `Config`
#[derive(Debug, Clone, PartialEq)]
pub enum OpStackConfigError {
    /// An error thrown during toml parsing
    Toml(figment::Error),
    /// Any other error thrown when constructing the config's figment
    Other(figment::Error),
}

impl fmt::Display for OpStackConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fmt_err = |err: &figment::Error, f: &mut fmt::Formatter<'_>| {
            write!(f, "{err}")?;
            if !err.path.is_empty() {
                // the path will contain the setting value like `["etherscan_api_key"]`
                write!(f, " for setting `{}`", err.path.join("."))?;
            }
            Ok(())
        };

        match self {
            OpStackConfigError::Toml(err) => {
                f.write_str("stack.toml error: ")?;
                fmt_err(err, f)
            }
            OpStackConfigError::Other(err) => {
                f.write_str("op stack config error: ")?;
                fmt_err(err, f)
            }
        }
    }
}

impl Error for OpStackConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            OpStackConfigError::Other(error) | OpStackConfigError::Toml(error) => {
                Error::source(error)
            }
        }
    }
}
