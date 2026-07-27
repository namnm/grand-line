use crate::prelude::*;
use chrono::{Duration, Utc};

pub fn now() -> DateTimeUtc {
    Utc::now()
}

pub const fn duration_w(weeks: i64) -> Duration {
    Duration::weeks(weeks)
}
pub const fn duration_d(days: i64) -> Duration {
    Duration::days(days)
}
pub const fn duration_h(hours: i64) -> Duration {
    Duration::hours(hours)
}
pub const fn duration_m(minutes: i64) -> Duration {
    Duration::minutes(minutes)
}
pub const fn duration_s(seconds: i64) -> Duration {
    Duration::seconds(seconds)
}
pub const fn duration_ms(ms: i64) -> Duration {
    Duration::milliseconds(ms)
}
