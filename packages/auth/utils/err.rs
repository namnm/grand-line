use crate::prelude::*;

#[grand_line_err]
pub enum MyErr {
    // ========================================================================
    // client errors
    //
    #[error("unauthenticated")]
    #[client]
    Unauthenticated,
    #[error("already authenticated")]
    #[client]
    AlreadyAuthenticated,

    #[error("this email address is already in use")]
    #[client]
    RegisterEmailExists,

    #[error("otp is expired or invalid")]
    #[client]
    OtpResolveInvalid,
    #[error("otp is not yet to re-request")]
    #[client]
    OtpReRequestTooSoon,

    #[error("email or password is incorrect")]
    #[client]
    LoginIncorrect,

    // ========================================================================
    // server errors
    //
    #[error("auth user impl not found")]
    UserImplNotFound,
}
