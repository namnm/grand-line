use grand_line::prelude::*;

#[model(by_id = false)]
pub struct User {
    pub email: String,
    #[graphql(skip)]
    pub password_hashed: String,
}
