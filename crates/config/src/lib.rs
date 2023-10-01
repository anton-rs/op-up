#![doc = include_str!("../README.md")]
#![feature(generic_const_exprs)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    rustdoc::all
)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use figment::Profile;
use serde::{Deserialize, Serialize};

use op_stack::components::{layer_one::L1Client, layer_two::L2Client, rollup::RollupClient};

/// RootPath convenience re-export
mod root;
pub use root::RootPath;

/// OP Stack Configuration
///
/// # Defaults
///
/// All configuration values have a default, documented in the [fields](#fields)
/// section below. [`Config::default()`] returns the default values for
/// the default profile while [`Config::with_root()`] returns the values based on the given
/// directory. [`Config::load()`] starts with the default profile and merges various providers into
/// the config, same for [`Config::load_with_root()`], but there the default values are determined
/// by [`Config::with_root()`]
///
/// # Provider Details
///
/// `Config` is a Figment [`Provider`] with the following characteristics:
///
///   * **Profile**
///
///     The profile is set to the value of the `profile` field.
///
///   * **Metadata**
///
///     This provider is named `OP Stack Config`. It does not specify a
///     [`Source`](figment::Source) and uses default interpolation.
///
///   * **Data**
///
///     The data emitted by this provider are the keys and values corresponding
///     to the fields and values of the structure. The dictionary is emitted to
///     the "default" meta-profile.
///
/// Note that these behaviors differ from those of [`Config::figment()`].
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Config {
    /// The selected profile. **(default: _default_ `default`)**
    ///
    /// **Note:** This field is never serialized nor deserialized. When a
    /// `Config` is merged into a `Figment` as a `Provider`, this profile is
    /// selected on the `Figment`. When a `Config` is extracted, this field is
    /// set to the extracting Figment's selected `Profile`.
    #[serde(skip)]
    pub profile: Profile,

    /// The type of L1 Client to use. **(default: _default_ `L1Client::Geth`)**
    pub l1_client: L1Client,
    /// The type of L2 Client to use. **(default: _default_ `L2Client::Geth`)**
    pub l2_client: L2Client,
    /// The type of Rollup Client to use. **(default: _default_ `RollupClient::Node`)**
    pub rollup_client: RollupClient,

    /// Enable Sequencing. **(default: _default_ `false`)**
    pub enable_sequencing: bool,
    /// Enable Fault Proofs. **(default: _default_ `false`)**
    pub enable_frault_proofs: bool,

    /// JWT secret that should be used for any rpc calls
    pub eth_rpc_jwt: Option<String>,

    /// The root path where the config detection started from, `Config::with_root`
    #[doc(hidden)]
    // Skip serialization here, so it won't be included in the [`Config::to_string()`]
    // representation, but will be deserialized from `Figment` so that commands can
    // override it.
    #[serde(rename = "root", default, skip_serializing)]
    pub __root: RootPath,
}
