use crate::prelude::*;

/// Implemented by the host app's user entity to plug it into the auth resolvers,
/// exposing the email and hashed password columns used for login, register, and forgot.
pub trait AuthUser
where
    Self: EntityX,
{
    /// Column holding the user's email, used to look up the row on login/register/forgot.
    fn email_col() -> Self::C;
    /// Column holding the user's hashed password.
    fn hashed_password_col() -> Self::C;
    fn get_email(m: &Self::M) -> &str;
    /// Reads the hashed password from a user model, compared against the login attempt.
    fn get_password_hashed(m: &Self::M) -> &str;
}
