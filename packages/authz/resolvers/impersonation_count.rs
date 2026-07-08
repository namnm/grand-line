use crate::prelude::*;

#[count(Impersonation, authz(realm = "org"))]
fn org_impersonation_count() {
    let org_id = ctx.authz().await?;
    Filter::default().add(ImpersonationColumn::OrgId.eq(org_id))
}

#[count(Impersonation, authz(realm = "system", skip_org))]
fn system_impersonation_count() {
}
