use op_config::Config;
use op_stack::{ChallengerAgent, L1Client, L2Client, RollupClient};

#[test]
fn test_default_config() {
    let config = Config::default();

    assert_eq!(config.l1_client, L1Client::default());
    assert_eq!(config.l2_client, L2Client::default());
    assert_eq!(config.rollup_client, RollupClient::default());
    assert_eq!(config.challenger, ChallengerAgent::default());
}
