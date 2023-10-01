use std::{fmt::Display, str::FromStr};

use enum_variants_strings::EnumVariantsStrings;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

/// Rollup Client
///
/// The OP Stack rollup client performs the derivation of the rollup state
/// from the L1 and L2 clients.
#[derive(Debug, Clone, PartialEq, EnumVariantsStrings, Deserialize, Serialize, EnumIter)]
#[enum_variants_strings_transform(transform = "kebab_case")]
pub enum RollupClient {
    /// OP Node
    OpNode,
    /// Magi
    Magi,
}

impl FromStr for RollupClient {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == RollupClient::OpNode.to_str() {
            return Ok(RollupClient::OpNode);
        }
        if s == RollupClient::Magi.to_str() {
            return Ok(RollupClient::Magi);
        }
        eyre::bail!("Invalid L2 client: {}", s)
    }
}

impl Display for RollupClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
