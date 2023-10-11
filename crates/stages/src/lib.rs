#![doc = include_str!("../README.md")]
#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    rustdoc::all
)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

pub(crate) mod git;
pub(crate) mod json;
pub(crate) mod net;

/// Stage
///
/// A stage is a synchronous step in the [Stages] executor that handles a component of the op stack.
pub trait Stage: std::fmt::Debug {
    /// Execute the stage.
    fn execute(&self) -> eyre::Result<()>;
}

/// Core Stages.
pub mod stages;
pub use stages::Stages;
