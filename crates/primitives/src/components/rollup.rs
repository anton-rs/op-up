use std::fmt::Display;

use enum_variants_strings::EnumVariantsStrings;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

/// Rollup Client
///
/// The OP Stack rollup client performs the derivation of the rollup state
/// from the L1 and L2 clients.
#[derive(Default, Clone, PartialEq, EnumVariantsStrings, Deserialize, Serialize, EnumIter)]
#[enum_variants_strings_transform(transform = "kebab_case")]
pub enum RollupClient {
    /// OP Node
    #[default]
    OpNode,
    /// Magi
    Magi,
}

impl std::fmt::Debug for RollupClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl std::str::FromStr for RollupClient {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == RollupClient::OpNode.to_str() {
            return Ok(RollupClient::OpNode);
        }
        if s == RollupClient::Magi.to_str() {
            return Ok(RollupClient::Magi);
        }
        eyre::bail!("Invalid rollup client: {}", s)
    }
}

impl Display for RollupClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_string() {
        assert_eq!(format!("{:?}", RollupClient::OpNode), "op-node");
        assert_eq!(format!("{:?}", RollupClient::Magi), "magi");
    }

    #[test]
    fn test_rollup_client_from_str() {
        assert_eq!(
            RollupClient::from_str("op-node").unwrap(),
            RollupClient::OpNode
        );
        assert_eq!(RollupClient::from_str("magi").unwrap(), RollupClient::Magi);
        assert!(RollupClient::from_str("invalid").is_err());
    }

    #[test]
    fn test_rollup_client_to_str() {
        assert_eq!(RollupClient::OpNode.to_str(), "op-node");
        assert_eq!(RollupClient::Magi.to_str(), "magi");
    }
}
