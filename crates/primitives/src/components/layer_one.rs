use std::fmt::Display;

use enum_variants_strings::EnumVariantsStrings;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

/// L1 Client
///
/// The OP Stack L1 client is an L1 execution client.
#[derive(Default, Clone, PartialEq, EnumVariantsStrings, Deserialize, Serialize, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[enum_variants_strings_transform(transform = "kebab_case")]
pub enum L1Client {
    /// Geth
    #[default]
    Geth,
    /// Erigon
    Erigon,
    /// Reth
    Reth,
}

impl std::fmt::Debug for L1Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl std::str::FromStr for L1Client {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == L1Client::Geth.to_str() {
            return Ok(L1Client::Geth);
        }
        if s == L1Client::Erigon.to_str() {
            return Ok(L1Client::Erigon);
        }
        if s == L1Client::Reth.to_str() {
            return Ok(L1Client::Reth);
        }
        eyre::bail!("Invalid L1 client: {}", s)
    }
}

impl Display for L1Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_string() {
        assert_eq!(
            serde_json::from_str::<L1Client>(r#""geth""#).unwrap(),
            L1Client::Geth
        );
        assert_eq!(
            serde_json::from_str::<L1Client>(r#""erigon""#).unwrap(),
            L1Client::Erigon
        );
        assert_eq!(
            serde_json::from_str::<L1Client>(r#""reth""#).unwrap(),
            L1Client::Reth
        );
        assert!(serde_json::from_str::<L1Client>(r#""invalid""#).is_err());
    }

    #[test]
    fn test_debug_string() {
        assert_eq!(format!("{:?}", L1Client::Geth), "geth");
        assert_eq!(format!("{:?}", L1Client::Erigon), "erigon");
        assert_eq!(format!("{:?}", L1Client::Reth), "reth");
    }

    #[test]
    fn test_l1_client_from_str() {
        assert_eq!("geth".parse::<L1Client>().unwrap(), L1Client::Geth);
        assert_eq!("erigon".parse::<L1Client>().unwrap(), L1Client::Erigon);
        assert_eq!("reth".parse::<L1Client>().unwrap(), L1Client::Reth);
        assert!("invalid".parse::<L1Client>().is_err());
    }

    #[test]
    fn test_l1_client_to_str() {
        assert_eq!(L1Client::Geth.to_str(), "geth");
        assert_eq!(L1Client::Erigon.to_str(), "erigon");
        assert_eq!(L1Client::Reth.to_str(), "reth");
    }
}
