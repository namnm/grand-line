use grand_line::prelude::*;

use crate::models::*;

/// Deletes the linked login session so the impersonation token stops working,
/// and soft-deletes the Impersonation record itself.
#[mutation(authz(realm = "org"))]
fn impersonate_revoke(id: String) -> ImpersonationGql {
    let org_id = ctx.authz().await?;
    let imp = Impersonation::find_by_id(&id).filter(ImpersonationColumn::OrgId.eq(&org_id)).one_or_404(tx).await?;

    LoginSession::delete_by_id(&imp.login_session_id).exec(tx).await?;
    Impersonation::soft_delete_by_id(&imp.id)?.exec(tx).await?;

    ImpersonationGql::from_id(&imp.id)
}
