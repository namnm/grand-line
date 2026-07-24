use grand_line::prelude::*;

use crate::am_ctx::*;
use crate::models::*;

#[gql_input]
pub struct RoleCreate {
    pub name: String,
    pub col_policy: JsonValue,
    pub row_policy: JsonValue,
}

#[create(Role, authz(realm = "org"))]
fn role_create() {
    let org_id = ctx.authz().await?;
    am_create!(Role {
        name: data.name,
        realm: "org".to_owned(),
        col_policy: data.col_policy,
        row_policy: data.row_policy,
        org_id: Some(org_id),
    })
}
