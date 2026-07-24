use grand_line::prelude::*;

/// App-level errors, all in one enum since this is application code, not a
/// shared library (no need to split per "package" the way grand_line's own
/// crates do).
#[grand_line_err]
pub enum SaasErr {
    #[error("unauthenticated")]
    #[client]
    Unauthenticated,
    #[error("already authenticated")]
    #[client]
    AlreadyAuthenticated,
    #[error("incorrect email or password")]
    #[client]
    LoginIncorrect,
    #[error("email already registered")]
    #[client]
    RegisterEmailExists,
    #[error("otp id/secret/code is invalid or expired")]
    #[client]
    OtpResolveInvalid,
    #[error("otp was already requested recently, try again later")]
    #[client]
    OtpReRequestTooSoon,
    #[error("the invitation email does not match the current user's email")]
    #[client]
    InvitationEmailMismatch,
}
