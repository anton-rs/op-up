use op_config::Config;
use op_primitives::{ChallengerAgent, L1Client, L2Client, RollupClient};
use std::path::PathBuf;
use temp_testdir::TempDir;

#[test]
fn test_default_config() {
    let config = Config::default();

    assert_eq!(config.artifacts, PathBuf::from(Config::STACK_DIR_NAME));

    assert_eq!(config.l1_client, L1Client::default());
    assert_eq!(config.l2_client, L2Client::default());
    assert_eq!(config.rollup_client, RollupClient::default());
    assert_eq!(config.challenger, ChallengerAgent::default());

    assert_eq!(config.enable_sequencing, false);
    assert_eq!(config.enable_fault_proofs, false);
}

#[test]
fn test_read_config_from_toml() {
    let tempdir = TempDir::default().permanent();
    std::env::set_current_dir(&tempdir).unwrap();

    std::fs::write(
        "stack.toml",
        r#"
        [default]
        l1-client = 'reth'
        l2-client = 'op-reth'
        rollup-client = 'magi'
        challenger = 'op-challenger-go'
        enable-sequencing = true
        enable-fault-proofs = true
        "#,
    )
    .unwrap();
    assert!(PathBuf::from("stack.toml").exists());

    let config = Config::from_toml("stack.toml").unwrap();
    assert_eq!(config.artifacts, PathBuf::from(Config::STACK_DIR_NAME));
    assert_eq!(config.l1_client, L1Client::Reth);
    assert_eq!(config.l2_client, L2Client::OpReth);
    assert_eq!(config.rollup_client, RollupClient::Magi);
    assert_eq!(config.challenger, ChallengerAgent::OpChallengerGo);
    assert_eq!(config.enable_sequencing, true);
    assert_eq!(config.enable_fault_proofs, true);
}

#[test]
fn test_create_artifacts_dir() {
    let tempdir = TempDir::default().permanent();
    std::env::set_current_dir(&tempdir).unwrap();

    let config = Config::default();
    config.create_artifacts_dir().unwrap();
    assert!(config.artifacts.exists());
    assert!(config.artifacts.is_dir());
}
