use crate::prelude::*;

#[detail(Impersonation, authz(realm = "org"))]
fn org_impersonation_detail() {
    let org_id = ctx.authz().await?;
    Filter::default().add(ImpersonationColumn::OrgId.eq(org_id))
}

#[detail(Impersonation, authz(realm = "system", skip_org))]
fn system_impersonation_detail() {
}
