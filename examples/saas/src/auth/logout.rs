use crate::prelude::*;

#[mutation(auth)]
fn logout() -> LoginSessionGql {
    let session_id = ctx.auth_session().await?;
    LoginSession::delete_by_id(&session_id).exec(tx).await?;
    LoginSessionGql::from_id(&session_id)
}
