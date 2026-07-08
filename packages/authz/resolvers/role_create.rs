use crate::prelude::*;

#[gql_input]
pub struct OrgRoleCreate {
    pub name: String,
    pub col_policy: JsonValue,
    pub row_policy: JsonValue,
}

#[create(Role, authz(realm = "org"))]
fn org_role_create() {
    let org_id = ctx.authz().await?;
    am_create!(Role {
        name: data.name,
        realm: "org".to_owned(),
        col_policy: data.col_policy,
        row_policy: data.row_policy,
        org_id: Some(org_id),
    })
}

#[gql_input]
pub struct SystemRoleCreate {
    pub name: String,
    pub realm: String,
    pub org_id: Option<String>,
    pub col_policy: JsonValue,
    pub row_policy: JsonValue,
}

#[create(Role, authz(realm = "system", skip_org))]
fn system_role_create() {
    am_create!(Role {
        name: data.name,
        realm: data.realm,
        col_policy: data.col_policy,
        row_policy: data.row_policy,
        org_id: data.org_id,
    })
}
