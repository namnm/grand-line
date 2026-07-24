use grand_line::prelude::*;

use crate::models::*;

#[query]
async fn login_session_current() -> Option<LoginSessionGql> {
    if let Some(ls) = current_login_session(ctx).await? {
        Some(ls.into_gql(ctx).await?)
    } else {
        None
    }
}
