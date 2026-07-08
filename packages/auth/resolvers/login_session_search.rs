use crate::prelude::*;

pub async fn login_session_search_impl(
    ctx: &Context<'_>,
    filter: Option<LoginSessionFilter>,
    order_by: Option<Vec<LoginSessionOrderBy>>,
    page: Option<Pagination>,
) -> Res<Vec<LoginSessionGql>> {
    ctx.auth_ensure_authenticated().await?;
    let tx = &*ctx.tx().await?;
    let extra = login_session_get_filter(ctx).await?;
    let default_order = order_by!(LoginSession[UpdatedAtDesc]);
    LoginSession::gql_search(
        ctx,
        tx,
        None,
        filter,
        Some(extra),
        order_by,
        Some(default_order),
        page,
        Some(false),
    )
    .await
}

pub async fn login_session_count_impl(ctx: &Context<'_>, filter: Option<LoginSessionFilter>) -> Res<u64> {
    ctx.auth_ensure_authenticated().await?;
    let tx = &*ctx.tx().await?;
    let extra = login_session_get_filter(ctx).await?;
    LoginSession::gql_count(tx, filter, Some(extra), Some(false)).await
}

async fn login_session_get_filter(ctx: &Context<'_>) -> Res<LoginSessionFilter> {
    let c = ctx.auth_config();
    let arc = ctx.auth_with_cache().await?;
    let ls = arc.as_ref().as_ref().ok_or(MyErr::Unauthenticated)?;
    let f = filter!(LoginSession {
        id_ne: ls.id.clone(),
        user_id: ls.user_id.clone(),
        created_at_gte: now() - duration_ms(c.cookie_login_session_expires_ms),
    });
    Ok(f)
}
