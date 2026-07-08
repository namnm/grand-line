use crate::prelude::*;

#[detail(Role, authz(realm = "org"))]
fn org_role_detail() {
    let org_id = ctx.authz().await?;
    Filter::default().add(RoleColumn::OrgId.eq(org_id))
}

#[detail(Role, authz(realm = "system", skip_org))]
fn system_role_detail() {
}
