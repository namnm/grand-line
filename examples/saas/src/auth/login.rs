use grand_line::prelude::*;

use crate::err::SaasErr;
use crate::models::*;

#[gql_input]
pub struct Login {
    pub email: String,
    pub password: String,
}

/// Validates email/password and creates a new login session, or returns
/// SaasErr::LoginIncorrect if the email is not found or the password does not match.
#[mutation]
fn login(data: Login) -> LoginSessionWithSecret {
    ensure_not_authenticated(ctx).await?;

    let u = User::find()
        .include_deleted(false)
        .filter(UserColumn::Email.eq(&data.email))
        .one(tx)
        .await?
        .ok_or(SaasErr::LoginIncorrect)?;

    if !rand_utils::password_eq(&u.password_hashed, &data.password) {
        return Err(SaasErr::LoginIncorrect.into());
    }

    login_session_create(ctx, tx, &u.id).await?
}
