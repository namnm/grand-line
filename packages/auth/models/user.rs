use crate::prelude::*;

pub trait AuthUser: EntityX + Send + Sync {
    fn email_col() -> Self::C;
    fn password_col() -> Self::C;
    fn get_email(m: &Self::M) -> &str;
    fn get_password_hashed(m: &Self::M) -> &str;
}
