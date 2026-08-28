use super::prelude::*;
use redis::RedisError;

/// Errors surfaced by the subscription package.
#[grand_line_err]
pub enum MyErr {
    // ------------------------------------------------------------------------
    // server errors
    // ------------------------------------------------------------------------
    #[error("subscription redis error: {inner}")]
    Redis {
        #[from]
        inner: RedisError,
    },
}

impl From<RedisError> for GrandLineErr {
    fn from(v: RedisError) -> Self {
        MyErr::from(v).into()
    }
}
