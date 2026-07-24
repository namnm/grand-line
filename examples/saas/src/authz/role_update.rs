use grand_line::prelude::*;

use crate::models::*;

#[gql_input]
pub struct RoleUpdate {
    pub name: String,
    pub col_policy: JsonValue,
    pub row_policy: JsonValue,
}

#[mutation(authz(realm = "org"))]
fn role_update(id: String, data: RoleUpdate) -> RoleGql {
    let org_id = ctx.authz().await?;

    Role::find_by_id(&id).filter(RoleColumn::OrgId.eq(&org_id)).exists_or_404(tx).await?;

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
