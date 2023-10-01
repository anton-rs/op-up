use std::{fmt::Display, str::FromStr};

use enum_variants_strings::EnumVariantsStrings;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

/// L2 Client
///
/// The OP Stack L2 client is a minimally modified version of the L1 client
/// that supports deposit transactions as well as a few other small OP-specific
/// changes.
#[derive(Debug, Clone, PartialEq, EnumVariantsStrings, Deserialize, Serialize, EnumIter)]
#[enum_variants_strings_transform(transform = "kebab_case")]
pub enum L2Client {
    /// OP Geth
    OpGeth,
    /// OP Erigon
    OpErigon,
    /// OP Reth
    OpReth,
}

impl FromStr for L2Client {
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
