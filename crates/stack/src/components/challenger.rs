use std::{fmt::Display, str::FromStr};

use enum_variants_strings::EnumVariantsStrings;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

/// Challenger Agent Implementations
#[derive(Debug, Clone, PartialEq, EnumVariantsStrings, Deserialize, Serialize, EnumIter)]
#[enum_variants_strings_transform(transform = "kebab_case")]
pub enum ChallengerAgent {
    /// A Go implementation of the challenger agent
    OpChallengerGo,
    /// A Rust implementation of the challenger agent
    OpChallengerRust,
}

impl FromStr for ChallengerAgent {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == ChallengerAgent::OpChallengerGo.to_str() {
            return Ok(ChallengerAgent::OpChallengerGo);
        }
        if s == ChallengerAgent::OpChallengerRust.to_str() {
            return Ok(ChallengerAgent::OpChallengerRust);
        }
        eyre::bail!("Invalid L2 client: {}", s)
    }
}

impl Display for ChallengerAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
