use crate::prelude::*;

#[search(Role, authz(realm = "org"))]
fn org_role_search() {
    let org_id = ctx.authz().await?;
    Search::default().add(RoleColumn::OrgId.eq(org_id))
}

#[search(Role, authz(realm = "system", skip_org))]
fn system_role_search() {
}
