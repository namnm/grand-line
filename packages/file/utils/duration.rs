use core::time::Duration;

/// Converts a millisecond count into a std Duration, negative values clamp to zero.
pub fn std_duration_ms(ms: i64) -> Duration {
    Duration::from_millis(ms.max(0).unsigned_abs())
}
