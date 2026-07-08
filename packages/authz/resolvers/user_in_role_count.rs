use crate::prelude::*;

#[count(UserInRole, authz(realm = "org"))]
fn org_user_in_role_count() {
    let org_id = ctx.authz().await?;
    Filter::default().add(UserInRoleColumn::OrgId.eq(org_id))
}

#[count(UserInRole, authz(realm = "system", skip_org))]
fn system_user_in_role_count() {
}
