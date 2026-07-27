use crate::prelude::*;

#[query]
async fn login_session_current() -> Option<LoginSessionGql> {
    let arc = ctx.auth_unchecked().await?;
    let Some(m) = arc.as_ref() else {
        return Ok(None);
    };

    let ls = LoginSession::find()
        .include_deleted(false)
        .filter_by_id(&m.id)
        .one(tx)
        .await?;

    match ls {
        Some(ls) => Some(ls.into_gql(ctx).await?),
        None => None,
    }
}
