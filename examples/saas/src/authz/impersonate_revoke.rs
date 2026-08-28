use crate::prelude::*;

#[mutation(check = authz_org)]
fn impersonate_revoke(id: String) -> ImpersonationGql {
    let imp = ctx.authz_org_one_or_404::<Impersonation>(&id).await?;

    LoginSession::delete_by_id(&imp.login_session_id).exec(db).await?;
    Impersonation::soft_delete_by_id(&imp.id)?.exec(db).await?;

    ImpersonationGql::from_id(&imp.id)
}
