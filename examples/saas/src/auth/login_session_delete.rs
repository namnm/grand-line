use crate::prelude::*;

#[mutation(auth)]
fn login_session_delete(id: String) -> LoginSessionGql {
    let user_id = ctx.auth().await?;
    let session_id = ctx.auth_session().await?;

    if id == session_id {
        return Err(SaasErr::SessionDeleteCurrent.into());
    }

    LoginSession::delete_by_id(&id)
        .filter(LoginSessionColumn::UserId.eq(&user_id))
        .exec(tx)
        .await?;

    LoginSessionGql::from_id(&id)
}

#[mutation(auth)]
fn login_session_delete_all() -> Vec<LoginSessionGql> {
    let user_id = ctx.auth().await?;
    let session_id = ctx.auth_session().await?;

    let r = LoginSession::find()
        .include_deleted(false)
        .filter(LoginSessionColumn::Id.ne(&session_id))
        .filter(LoginSessionColumn::UserId.eq(&user_id))
        .gql_select_id()
        .all(tx)
        .await?;

    LoginSession::delete_many()
        .filter(LoginSessionColumn::Id.ne(&session_id))
        .filter(LoginSessionColumn::UserId.eq(&user_id))
        .exec(tx)
        .await?;

    r
}
