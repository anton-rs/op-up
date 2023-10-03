#![doc = include_str!("../README.md")]
#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    rustdoc::all
)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

/// OP Stack [Config].
pub mod stack;
pub use stack::*;

/// [StageConfig] is a [figment::Provider] that holds the [Stage] configuration.
pub mod stage;
pub use stage::*;

/// Convenience [figment::Error] wrapper.
/// Uses a custom [OpStackConfigError] under the hood.
pub(crate) mod error;

/// Holds a [figment::Provider] that is used to retrieve a toml file.
pub(crate) mod toml;

/// Renames the [figment::Provider] `from` key to `to`.
pub(crate) mod rename;

/// Wraps a profile increasing the key depth.
pub(crate) mod wraps;

/// Extends a [figment::Provider] by warning about deprecated profile key usage.
pub(crate) mod optional;

/// Unwraps a profile reducing the key depth.
pub(crate) mod unwraps;

/// A wrapper for the root path used during toml config detection.
pub mod root;
pub use root::*;
