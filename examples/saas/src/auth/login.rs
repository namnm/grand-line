use crate::prelude::*;

#[gql_input]
pub struct Login {
    pub email: Email,
    pub password: String,
}

#[mutation(check = unauthenticated)]
fn login(data: Login) -> LoginSessionWithSecret {
    let u = User::find()
        .include_deleted(false)
        .filter(UserColumn::Email.eq(&data.email.0))
        .one(db)
        .await?
        .ok_or(SaasErr::LoginIncorrect)?;

    if !rand_utils::password_eq(&u.password_hashed, &data.password) {
        return Err(SaasErr::LoginIncorrect.into());
    }

    login_session_create(ctx, db, &u.id).await?
}
