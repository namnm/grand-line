use crate::prelude::*;

#[gql_input]
pub struct RoleUpdate {
    pub name: String,
    pub col_policy: JsonValue,
    pub row_policy: JsonValue,
}

#[mutation(check = authz_org)]
fn role_update(id: String, data: RoleUpdate) -> RoleGql {
    ctx.authz_org_one_or_404::<Role>(&id).await?;

    am_update!(Role {
        id: id.clone(),
        name: data.name,
        col_policy: data.col_policy,
        row_policy: data.row_policy,
    })
    .exec_without_ctx(db)
    .await?;

    RoleGql::from_id(&id)
}
