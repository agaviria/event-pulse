use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current timestamp in microseconds since the Unix epoch.
pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!")
        .as_micros() as u64
}
