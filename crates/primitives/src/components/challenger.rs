use std::fmt::Display;

use enum_variants_strings::EnumVariantsStrings;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

/// Challenger Agent Implementations
#[derive(Default, Clone, PartialEq, EnumVariantsStrings, Deserialize, Serialize, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[enum_variants_strings_transform(transform = "kebab_case")]
pub enum ChallengerAgent {
    /// A Go implementation of the challenger agent
    #[default]
    OpChallengerGo,
    /// A Rust implementation of the challenger agent
    OpChallengerRust,
}

impl std::fmt::Debug for ChallengerAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl std::str::FromStr for ChallengerAgent {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == ChallengerAgent::OpChallengerGo.to_str() {
            return Ok(ChallengerAgent::OpChallengerGo);
        }
        if s == ChallengerAgent::OpChallengerRust.to_str() {
            return Ok(ChallengerAgent::OpChallengerRust);
        }
        eyre::bail!("Invalid challenger agent: {}", s)
    }
}

impl Display for ChallengerAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_string() {
        assert_eq!(
            serde_json::from_str::<ChallengerAgent>(r#""op-challenger-go""#).unwrap(),
            ChallengerAgent::OpChallengerGo
        );
        assert_eq!(
            serde_json::from_str::<ChallengerAgent>(r#""op-challenger-rust""#).unwrap(),
            ChallengerAgent::OpChallengerRust
        );
    }

    #[test]
    fn test_debug_string() {
        assert_eq!(
            format!("{:?}", ChallengerAgent::OpChallengerGo),
            "op-challenger-go"
        );
        assert_eq!(
            format!("{:?}", ChallengerAgent::OpChallengerRust),
            "op-challenger-rust"
        );
    }

    #[test]
    fn test_challenger_agent_from_str() {
        assert_eq!(
            "op-challenger-go".parse::<ChallengerAgent>().unwrap(),
            ChallengerAgent::OpChallengerGo
        );
        assert_eq!(
            "op-challenger-rust".parse::<ChallengerAgent>().unwrap(),
            ChallengerAgent::OpChallengerRust
        );
        assert!("invalid".parse::<ChallengerAgent>().is_err());
    }

    #[test]
    fn test_challenger_agent_display() {
        assert_eq!(
            format!("{}", ChallengerAgent::OpChallengerGo),
            "op-challenger-go"
        );
        assert_eq!(
            format!("{}", ChallengerAgent::OpChallengerRust),
            "op-challenger-rust"
        );
    }
}
