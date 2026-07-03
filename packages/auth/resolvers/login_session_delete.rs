use crate::prelude::*;

#[mutation(auth)]
fn login_session_delete(id: String) -> LoginSessionGql {
    ctx.auth_ensure_authenticated().await?;

    let tx = &*ctx.tx().await?;
    let arc = ctx.auth_unchecked().await?;
    let ls = arc.as_ref().as_ref().ok_or(MyErr::Unauthenticated)?;

    LoginSession::delete_by_id(&id)
        .filter(LoginSessionColumn::UserId.eq(&ls.user_id))
        .exec(tx)
        .await?;

    LoginSessionGql::from_id(&id)
}

#[mutation(auth)]
fn login_session_delete_all() -> Vec<LoginSessionGql> {
    ctx.auth_ensure_authenticated().await?;

    let tx = &*ctx.tx().await?;
    let arc = ctx.auth_unchecked().await?;
    let ls = arc.as_ref().as_ref().ok_or(MyErr::Unauthenticated)?;

    let r = LoginSession::find()
        .include_deleted(false)
        .filter(LoginSessionColumn::Id.ne(&ls.id))
        .filter(LoginSessionColumn::UserId.eq(&ls.user_id))
        .gql_select_id()
        .all(tx)
        .await?;

    LoginSession::delete_many()
        .filter(LoginSessionColumn::Id.ne(&ls.id))
        .filter(LoginSessionColumn::UserId.eq(&ls.user_id))
        .exec(tx)
        .await?;

    r
}
