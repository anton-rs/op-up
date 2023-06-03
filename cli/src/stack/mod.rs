pub mod l1_client;
pub use l1_client::{L1Client, ERIGON, GETH};

pub mod l2_client;
pub use l2_client::{L2Client, OP_ERIGON, OP_GETH};

pub mod rollup_client;
pub use rollup_client::{RollupClient, MAGI, OP_NODE};

pub mod challenger_agent;
pub use challenger_agent::{ChallengerAgent, OP_CHALLENGER_GO, OP_CHALLENGER_RUST};
