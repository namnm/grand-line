use crate::prelude::*;

#[count(Role, authz(realm = "org"))]
fn org_role_count() {
    let org_id = ctx.authz().await?;
    Filter::default().add(RoleColumn::OrgId.eq(org_id))
}

#[count(Role, authz(realm = "system", skip_org))]
fn system_role_count() {
}
