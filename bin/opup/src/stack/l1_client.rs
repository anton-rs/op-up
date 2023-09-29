use std::{fmt::Display, str::FromStr};

use eyre::{bail, Report};

// L1 clients
pub const GETH: &str = "geth";
pub const ERIGON: &str = "erigon";

pub enum L1Client {
    Geth,
    Erigon,
}

impl FromStr for L1Client {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            GETH => Ok(L1Client::Geth),
            ERIGON => Ok(L1Client::Erigon),
            _ => bail!("Invalid L1 client: {}", s),
        }
    }
}

impl Display for L1Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            L1Client::Geth => write!(f, "{}", GETH),
            L1Client::Erigon => write!(f, "{}", ERIGON),
        }
    }
}
