use crate::prelude::*;

#[search(Impersonation, authz(realm = "org"))]
fn org_impersonation_search() {
    let org_id = ctx.authz().await?;
    Search::default().add(ImpersonationColumn::OrgId.eq(org_id))
}

#[search(Impersonation, authz(realm = "system", skip_org))]
fn system_impersonation_search() {
}
