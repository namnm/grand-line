use grand_line::prelude::*;

use crate::models::*;

#[mutation]
fn login_session_delete(id: String) -> LoginSessionGql {
    let ls = ensure_authenticated(ctx).await?;

    LoginSession::delete_by_id(&id)
        .filter(LoginSessionColumn::UserId.eq(&ls.user_id))
        .exec(tx)
        .await?;

    LoginSessionGql::from_id(&id)
}

#[mutation]
fn login_session_delete_all() -> Vec<LoginSessionGql> {
    let ls = ensure_authenticated(ctx).await?;

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
