use crate::prelude::*;

/// Errors surfaced by the app, split into client-facing and server-only variants.
#[grand_line_err]
pub enum SaasErr {
    #[error("incorrect email or password")]
    #[client]
    LoginIncorrect,
    #[error("email already registered")]
    #[client]
    RegisterEmailExists,
    #[error("the invitation email does not match the current user's email")]
    #[client]
    InvitationEmailMismatch,
    #[error("cannot delete the current session")]
    #[client]
    SessionDeleteCurrent,
}
