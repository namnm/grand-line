use crate::prelude::*;

#[search(UserInRole, authz(realm = "org"))]
fn org_user_in_role_search() {
    let org_id = ctx.authz().await?;
    Search::default().add(UserInRoleColumn::OrgId.eq(org_id))
}

#[search(UserInRole, authz(realm = "system", skip_org))]
fn system_user_in_role_search() {
}
