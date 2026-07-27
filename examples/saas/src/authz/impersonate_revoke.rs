use crate::prelude::*;

#[mutation(authz(realm = "org"))]
fn impersonate_revoke(id: String) -> ImpersonationGql {
    let imp = ctx.authz_org_one_or_404::<Impersonation>(&id).await?;

    LoginSession::delete_by_id(&imp.login_session_id).exec(tx).await?;
    Impersonation::soft_delete_by_id(&imp.id)?.exec(tx).await?;

    ImpersonationGql::from_id(&imp.id)
}
