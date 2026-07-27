use crate::prelude::*;
use argon2::password_hash::Error as PasswordHashErr;
use serde_qs::Error as QsErr;

/// Errors surfaced by the rand utils package, split into client-facing and server-only variants.
#[grand_line_err]
pub enum MyErr {
    // ------------------------------------------------------------------------
    // client errors
    // ------------------------------------------------------------------------
    #[error("password is too weak or invalid")]
    #[client]
    PasswordInvalid,

    // ------------------------------------------------------------------------
    // server errors
    // ------------------------------------------------------------------------
    #[error("hash password error: {inner}")]
    PasswordHash {
        inner: PasswordHashErr,
    },
    #[error("query string error: {inner}")]
    QsErr {
        inner: QsErr,
    },
    #[error("hmac error: {inner}")]
    HmacErr {
        inner: String,
    },
}

impl From<PasswordHashErr> for MyErr {
    fn from(v: PasswordHashErr) -> Self {
        Self::PasswordHash {
            inner: v,
        }
    }
}

impl From<QsErr> for MyErr {
    fn from(v: QsErr) -> Self {
        Self::QsErr {
            inner: v,
        }
    }
}
