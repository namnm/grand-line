use grand_line::prelude::*;

use crate::models::*;

#[mutation]
fn logout() -> LoginSessionGql {
    let ls = ensure_authenticated(ctx).await?;
    LoginSession::delete_by_id(&ls.id).exec(tx).await?;
    LoginSessionGql::from_id(&ls.id)
}
