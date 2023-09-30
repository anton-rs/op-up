use std::{fmt::Display, str::FromStr};

use eyre::{bail, Report};

// Rollup clients
pub const OP_NODE: &str = "op-node";
pub const MAGI: &str = "magi";

pub enum RollupClient {
    OpNode,
    Magi,
}

impl FromStr for RollupClient {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            OP_NODE => Ok(RollupClient::OpNode),
            MAGI => Ok(RollupClient::Magi),
            _ => bail!("Invalid rollup client: {}", s),
        }
    }
}

impl Display for RollupClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RollupClient::OpNode => write!(f, "{}", OP_NODE),
            RollupClient::Magi => write!(f, "{}", MAGI),
        }
    }
}
