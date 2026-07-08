use crate::prelude::*;

#[detail(UserInRole, authz(realm = "org"))]
fn org_user_in_role_detail() {
    let org_id = ctx.authz().await?;
    Filter::default().add(UserInRoleColumn::OrgId.eq(org_id))
}

#[detail(UserInRole, authz(realm = "system", skip_org))]
fn system_user_in_role_detail() {
}
