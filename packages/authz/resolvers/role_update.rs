use crate::prelude::*;

#[gql_input]
pub struct RoleUpdate {
    pub name: String,
    pub col_policy: JsonValue,
    pub row_policy: JsonValue,
}

#[mutation(authz(realm = "org"))]
fn org_role_update(id: String, data: RoleUpdate) -> RoleGql {
    let org_id = ctx.authz().await?;

    let tx = &*ctx.tx().await?;
    Role::find_by_id(&id)
        .filter(RoleColumn::OrgId.eq(&org_id))
        .exists_or_404(tx)
        .await?;

    am_update!(Role {
        id: id.clone(),
        name: data.name,
        col_policy: data.col_policy,
        row_policy: data.row_policy,
    })
    .exec_without_ctx(tx)
    .await?;

    RoleGql::from_id(&id)
}

#[update(Role, authz(realm = "system", skip_org), resolver_inputs)]
fn system_role_update(id: String, data: RoleUpdate) {
    am_update!(Role {
        id: id.clone(),
        name: data.name,
        col_policy: data.col_policy,
        row_policy: data.row_policy,
    })
}
