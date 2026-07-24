use grand_line::prelude::*;

use crate::models::*;

#[detail(Role, authz(realm = "org"))]
fn role_detail() {
    let org_id = ctx.authz().await?;
    Filter::default().add(RoleColumn::OrgId.eq(org_id))
}
