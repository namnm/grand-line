use crate::prelude::*;

pub trait AuthUser
where
    Self: EntityX,
{
    fn email_col() -> Self::C;
    fn hashed_password_col() -> Self::C;
    fn get_email(m: &Self::M) -> &str;
    fn get_password_hashed(m: &Self::M) -> &str;
}
