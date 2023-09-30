use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Returns the current timestamp in seconds.
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
