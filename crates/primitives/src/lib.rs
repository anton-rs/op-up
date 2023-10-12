#![doc = include_str!("../README.md")]
#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    rustdoc::all
)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[doc(hidden)]
pub mod artifacts;
pub use artifacts::*;

#[doc(hidden)]
pub mod genesis;
pub use genesis::*;

mod monorepo;
pub use monorepo::*;

/// Core components of the OP Stack
pub mod components;
pub use components::{
    challenger::ChallengerAgent, layer_one::L1Client, layer_two::L2Client, rollup::RollupClient,
};
