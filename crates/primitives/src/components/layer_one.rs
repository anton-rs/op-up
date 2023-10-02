use std::{fmt::Display, str::FromStr};

use enum_variants_strings::EnumVariantsStrings;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

/// L1 Client
///
/// The OP Stack L1 client is an L1 execution client.
#[derive(
    Default, Debug, Clone, PartialEq, EnumVariantsStrings, Deserialize, Serialize, EnumIter,
)]
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

impl FromStr for L1Client {
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
