use crate::prelude::*;

#[mutation(authz(realm = "org"))]
fn org_impersonate_revoke(id: String) -> ImpersonationGql {
    let org_id = ctx.authz().await?;

    let tx = &*ctx.tx().await?;
    let imp = Impersonation::find_by_id(&id)
        .filter(ImpersonationColumn::OrgId.eq(&org_id))
        .one_or_404(tx)
        .await?;

    revoke_impl(ctx, tx, imp).await?
}

#[mutation(authz(realm = "system", skip_org))]
fn system_impersonate_revoke(id: String) -> ImpersonationGql {
    let tx = &*ctx.tx().await?;
    let imp = Impersonation::find_by_id(&id).one_or_404(tx).await?;

    revoke_impl(ctx, tx, imp).await?
}

/// Deletes the linked login session so the impersonation token stops working,
/// and soft-deletes the Impersonation record itself.
async fn revoke_impl(ctx: &Context<'_>, tx: &DatabaseTransaction, imp: ImpersonationSql) -> Res<ImpersonationGql> {
    let h = &ctx.authz_config().handlers;

    LoginSession::delete_by_id(&imp.login_session_id).exec(tx).await?;
    Impersonation::soft_delete_by_id(&imp.id)?.exec(tx).await?;

    h.on_impersonate_revoke(ctx, &imp.id).await?;

    Ok(ImpersonationGql::from_id(&imp.id))
}
