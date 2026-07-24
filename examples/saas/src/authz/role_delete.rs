use grand_line::prelude::*;

use crate::models::*;

#[mutation(authz(realm = "org"))]
fn role_delete(id: String) -> RoleGql {
    let org_id = ctx.authz().await?;

    Role::find_by_id(&id).filter(RoleColumn::OrgId.eq(&org_id)).exists_or_404(tx).await?;
    Role::soft_delete_by_id(&id)?.filter(RoleColumn::OrgId.eq(&org_id)).exec(tx).await?;

    RoleGql::from_id(&id)
}
