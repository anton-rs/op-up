use std::{fmt::Display, str::FromStr};

use eyre::{bail, Report};

// L2 clients
pub const OP_GETH: &str = "op-geth";
pub const OP_ERIGON: &str = "op-erigon";

pub enum L2Client {
    OpGeth,
    OpErigon,
}

impl FromStr for L2Client {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            OP_GETH => Ok(L2Client::OpGeth),
            OP_ERIGON => Ok(L2Client::OpErigon),
            _ => bail!("Invalid L2 client: {}", s),
        }
    }
}

impl Display for L2Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            L2Client::OpGeth => write!(f, "{}", OP_GETH),
            L2Client::OpErigon => write!(f, "{}", OP_ERIGON),
        }
    }
}
