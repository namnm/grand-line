use grand_line::prelude::*;

use crate::models::*;

#[search(UserInRole, authz(realm = "org"))]
fn user_in_role_search() {
    let org_id = ctx.authz().await?;
    Search::default().add(UserInRoleColumn::OrgId.eq(org_id))
}

#[count(UserInRole, authz(realm = "org"))]
fn user_in_role_count() {
    let org_id = ctx.authz().await?;
    Filter::default().add(UserInRoleColumn::OrgId.eq(org_id))
}
