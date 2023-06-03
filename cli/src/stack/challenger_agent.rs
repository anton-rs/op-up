use std::{fmt::Display, str::FromStr};

use eyre::{bail, Report};

// Challenger agents
pub const OP_CHALLENGER_GO: &str = "op-challenger-go (go)";
pub const OP_CHALLENGER_RUST: &str = "op-challenger-rust (rust)";

pub enum ChallengerAgent {
    OpChallengerGo,
    OpChallengerRust,
}

impl FromStr for ChallengerAgent {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            OP_CHALLENGER_GO => Ok(ChallengerAgent::OpChallengerGo),
            OP_CHALLENGER_RUST => Ok(ChallengerAgent::OpChallengerRust),
            _ => bail!("Invalid challenger agent: {}", s),
        }
    }
}

impl Display for ChallengerAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChallengerAgent::OpChallengerGo => write!(f, "{}", OP_CHALLENGER_GO),
            ChallengerAgent::OpChallengerRust => write!(f, "{}", OP_CHALLENGER_RUST),
        }
    }
}
