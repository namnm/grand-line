use crate::prelude::*;

#[search(LoginSession, auth, include_deleted = false)]
fn resolver() {
    let f = get_filter(ctx).await?;
    let o = order_by!(LoginSession[UpdatedAtDesc]);
    (f, o).into()
}

#[count(LoginSession, auth, include_deleted = false)]
fn resolver() {
    let f = get_filter(ctx).await?;
    f.into()
}

async fn get_filter(ctx: &Context<'_>) -> Res<LoginSessionFilter> {
    let user_id = ctx.auth().await?;
    let session_id = ctx.auth_session().await?;
    let f = filter!(LoginSession {
        id_ne: session_id,
        user_id: user_id,
        created_at_gte: now() - duration_ms(ctx.auth_config().cookie_login_session_expires_ms),
    });
    Ok(f)
}
