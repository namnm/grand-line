use crate::prelude::*;

#[search(LoginSession, include_deleted = false, auth, authz_row = false)]
fn resolver() {
    ctx.auth_ensure_authenticated().await?;
    let f = get_filter(ctx).await?;
    let o = order_by!(LoginSession[UpdatedAtDesc]);
    (f, o).into()
}

#[count(LoginSession, include_deleted = false, auth, authz_row = false)]
fn resolver() {
    let f = get_filter(ctx).await?;
    f.into()
}

async fn get_filter(ctx: &Context<'_>) -> Res<LoginSessionFilter> {
    let c = ctx.auth_config();
    let arc = ctx.auth_unchecked().await?;
    let ls = arc.as_ref().as_ref().ok_or(MyErr::Unauthenticated)?;
    let f = filter!(LoginSession {
        id_ne: ls.id.clone(),
        user_id: ls.user_id.clone(),
        created_at_gte: now() - duration_ms(c.cookie_login_session_expires_ms),
    });
    Ok(f)
}
