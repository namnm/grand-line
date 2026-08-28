use crate::prelude::*;

#[mutation(check = authenticated)]
fn logout() -> LoginSessionGql {
    let session_id = ctx.auth_session().await?;
    LoginSession::delete_by_id(&session_id).exec(db).await?;
    LoginSessionGql::from_id(&session_id)
}
