use grand_line::prelude::*;

use crate::models::*;

#[search(LoginSession, include_deleted = false)]
fn resolver() {
    let ls = ensure_authenticated(ctx).await?;
    let f = filter!(LoginSession {
        id_ne: ls.id.clone(),
        user_id: ls.user_id.clone(),
        created_at_gte: now() - duration_ms(LOGIN_SESSION_COOKIE_EXPIRES_MS),
    });
    let o = order_by!(LoginSession [UpdatedAtDesc]);
    (f, o).into()
}

#[count(LoginSession, include_deleted = false)]
fn resolver() {
    let ls = ensure_authenticated(ctx).await?;
    filter!(LoginSession {
        id_ne: ls.id.clone(),
        user_id: ls.user_id.clone(),
        created_at_gte: now() - duration_ms(LOGIN_SESSION_COOKIE_EXPIRES_MS),
    })
    .into()
}
