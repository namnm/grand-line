use crate::prelude::*;

#[gql_input]
pub struct UserInRoleCreate {
    pub user_id: String,
    pub role_id: String,
}

#[create(UserInRole, check = authz_org)]
fn resolver() {
    let org_id = ctx.authz().await?;
    am_create!(UserInRole {
        user_id: data.user_id,
        role_id: data.role_id,
        org_id: Some(org_id),
    })
}
