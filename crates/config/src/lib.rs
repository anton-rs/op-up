#![doc = include_str!("../README.md")]
#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    rustdoc::all
)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

/// Stack Configuration
///
/// The [stack] module contains the core [Config] struct which is
/// responsible for holding the configuration of the OP stack.
///
/// ## Example
///
/// ```rust
/// use std::path::PathBuf;
/// use op_config::Config;
///
/// let config = Config::default();
/// assert_eq!(config.artifacts, PathBuf::from(Config::STACK_DIR_NAME));
/// ```
pub mod stack;
pub use stack::*;

/// [StageConfig] is a [figment::Provider] that holds the [Stage] configuration.
pub mod stages;

/// Providers are [figment::Provider]s used to retrieve configuration.
pub mod providers;

/// A wrapper for the root path used during toml config detection.
pub mod root;
