use crate::prelude::*;

/// Errors surfaced by the auth package, split into client-facing and server-only variants.
#[grand_line_err]
pub enum MyErr {
    // ------------------------------------------------------------------------
    // client errors
    // ------------------------------------------------------------------------
    #[error("unauthenticated")]
    #[client]
    Unauthenticated,
    #[error("already authenticated")]
    #[client]
    AlreadyAuthenticated,
    #[error("otp is expired or invalid")]
    #[client]
    OtpResolveInvalid,
    #[error("otp is not yet to re-request")]
    #[client]
    OtpReRequestTooSoon,

    // ------------------------------------------------------------------------
    // server errors
    // ------------------------------------------------------------------------
    #[error("auth session impl not found")]
    SessionImplNotFound,
    #[error("auth otp impl not found")]
    OtpImplNotFound,
}
