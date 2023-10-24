use std::fmt::Display;

use enum_variants_strings::EnumVariantsStrings;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

/// L2 Client
///
/// The OP Stack L2 client is a minimally modified version of the L1 client
/// that supports deposit transactions as well as a few other small OP-specific
/// changes.
#[derive(
    Default, Copy, Clone, PartialEq, EnumVariantsStrings, Deserialize, Serialize, EnumIter,
)]
#[serde(rename_all = "kebab-case")]
#[enum_variants_strings_transform(transform = "kebab_case")]
pub enum L2Client {
    /// OP Geth
    #[default]
    OpGeth,
    /// OP Erigon
    OpErigon,
    /// OP Reth
    OpReth,
}

impl std::fmt::Debug for L2Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl std::str::FromStr for L2Client {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == L2Client::OpGeth.to_str() {
            return Ok(L2Client::OpGeth);
        }
        if s == L2Client::OpErigon.to_str() {
            return Ok(L2Client::OpErigon);
        }
        if s == L2Client::OpReth.to_str() {
            return Ok(L2Client::OpReth);
        }
        eyre::bail!("Invalid L2 client: {}", s)
    }
}

impl Display for L2Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        assert_eq!(
            serde_json::from_str::<L2Client>("\"op-geth\"").unwrap(),
            L2Client::OpGeth
        );
        assert_eq!(
            serde_json::from_str::<L2Client>("\"op-erigon\"").unwrap(),
            L2Client::OpErigon
        );
        assert_eq!(
            serde_json::from_str::<L2Client>("\"op-reth\"").unwrap(),
            L2Client::OpReth
        );
        assert!(serde_json::from_str::<L2Client>("\"invalid\"").is_err());
    }

    #[test]
    fn test_debug_string() {
        assert_eq!(format!("{:?}", L2Client::OpGeth), "op-geth");
        assert_eq!(format!("{:?}", L2Client::OpErigon), "op-erigon");
        assert_eq!(format!("{:?}", L2Client::OpReth), "op-reth");
    }

    #[test]
    fn test_l2_client_from_str() {
        assert_eq!("op-geth".parse::<L2Client>().unwrap(), L2Client::OpGeth);
        assert_eq!("op-erigon".parse::<L2Client>().unwrap(), L2Client::OpErigon);
        assert_eq!("op-reth".parse::<L2Client>().unwrap(), L2Client::OpReth);
        assert!("invalid".parse::<L2Client>().is_err());
    }

    #[test]
    fn test_l2_client_display() {
        assert_eq!(L2Client::OpGeth.to_string(), "op-geth");
        assert_eq!(L2Client::OpErigon.to_string(), "op-erigon");
        assert_eq!(L2Client::OpReth.to_string(), "op-reth");
    }
}
