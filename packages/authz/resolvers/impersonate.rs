use crate::prelude::*;

#[mutation(authz(realm = "org"))]
fn org_impersonate(user_id: String, reason: String) -> LoginSessionWithSecret {
    let org_id = ctx.authz().await?;

    let tx = &*ctx.tx().await?;
    UserInRole::find()
        .filter(UserInRoleColumn::UserId.eq(&user_id))
        .filter(UserInRoleColumn::OrgId.eq(&org_id))
        .exists_or_404(tx)
        .await?;

    impersonate_impl(ctx, user_id, Some(org_id), reason).await?
}

/// Impersonates user_id on behalf of a system admin, requires U only to check the
/// target user exists, called from AuthzUserImplMutation<U> since it needs U: AuthUser.
pub async fn system_impersonate_impl<U>(
    ctx: &Context<'_>,
    user_id: String,
    reason: String,
) -> Res<LoginSessionWithSecret>
where
    U: AuthUser,
{
    ctx.authz_ensure_in_macro(AuthzEnsure {
        realm: "system".to_owned(),
        org: false,
        user: true,
    })
    .await?;

    let tx = &*ctx.tx().await?;
    U::find().filter(U::col_id().eq(&user_id)).exists_or_404(tx).await?;

    impersonate_impl(ctx, user_id, None, reason).await
}

/// Creates a login session for user_id without setting a cookie (the secret is
/// returned to the caller, who uses it via the Authorization header instead, so
/// the admin's own cookie-based session is unaffected), and records an
/// Impersonation row via the ctx-aware exec so created_by_id (the current admin)
/// is tracked automatically.
async fn impersonate_impl(
    ctx: &Context<'_>,
    user_id: String,
    org_id: Option<String>,
    reason: String,
) -> Res<LoginSessionWithSecret> {
    let tx = &*ctx.tx().await?;
    let lsd = ctx.login_session_data()?;
    let h = &ctx.authz_config().handlers;

    let secret = rand_utils::secret();
    let ls = am_create!(LoginSession {
        user_id: user_id.clone(),
        secret_hashed: rand_utils::secret_hash(&secret),
        ip: lsd.ip,
        ua: lsd.ua.to_json()?,
    })
    .exec_without_ctx(tx)
    .await?;

    let imp = am_create!(Impersonation {
        login_session_id: ls.id.clone(),
        user_id,
        org_id,
        reason,
    })
    .exec(ctx)
    .await?;

    h.on_impersonate(ctx, &imp.id).await?;

    Ok(LoginSessionWithSecret {
        inner: ls,
        secret,
    })
}
