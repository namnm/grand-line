use grand_line::prelude::*;

use crate::models::*;

#[detail(UserInRole, authz(realm = "org"))]
fn user_in_role_detail() {
    let org_id = ctx.authz().await?;
    Filter::default().add(UserInRoleColumn::OrgId.eq(org_id))
}
