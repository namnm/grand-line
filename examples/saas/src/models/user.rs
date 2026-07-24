use grand_line::prelude::*;

/// An app user with an email/password login.
#[model(by_id = false)]
pub struct User {
    pub email: String,
    #[graphql(skip)]
    pub password_hashed: String,
}
