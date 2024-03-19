use chrono::{DateTime, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current timestamp in microseconds since the Unix epoch.
pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!")
        .as_micros() as u64
}

/// Gets a chrono::DateTime<chrono::Utc> datetime.
pub fn get_current_datetime_utc() -> DateTime<Utc> {
    Utc::now()
}
