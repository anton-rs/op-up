/// Convenience [figment::Error] wrapper.
/// Uses a custom [OpStackConfigError] under the hood.
pub mod error;

/// Extends a [figment::Provider] by warning about deprecated profile key usage.
pub mod optional;

/// Renames the [figment::Provider] `from` key to `to`.
pub mod rename;

/// Holds a [figment::Provider] that is used to retrieve a toml file.
pub mod toml;

/// Unwraps a profile reducing the key depth.
pub mod unwraps;

/// Wraps a profile increasing the key depth.
pub mod wraps;
