use op_config::Config;
use op_primitives::{ChallengerAgent, L1Client, L2Client, RollupClient};
use std::path::PathBuf;

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
fn test_create_artifacts_dir() {
    // Create a temporary directory and set it as the current working directory.
    // This directory will be deleted when the `tmpdir` variable goes out of scope.
    let tmpdir = tempfile::tempdir().unwrap();
    std::env::set_current_dir(&tmpdir).unwrap();

    // Create a default configuration and create the artifact directory.
    let config = Config::default();
    config.create_artifacts_dir().unwrap();
    assert!(config.artifacts.exists());
    assert!(config.artifacts.is_dir());

    // Drop the `tmpdir` variable, which deletes the temporary directory.
    drop(tmpdir);

    // The temporary directory should no longer exist.
    assert!(!config.artifacts.exists());
}
