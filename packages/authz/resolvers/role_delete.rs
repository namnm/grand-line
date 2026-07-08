use crate::prelude::*;

#[mutation(authz(realm = "org"))]
fn org_role_delete(id: String) -> RoleGql {
    let org_id = ctx.authz().await?;

    let tx = &*ctx.tx().await?;
    Role::find_by_id(&id)
        .filter(RoleColumn::OrgId.eq(&org_id))
        .exists_or_404(tx)
        .await?;
    Role::soft_delete_by_id(&id)?
        .filter(RoleColumn::OrgId.eq(&org_id))
        .exec(tx)
        .await?;

    RoleGql::from_id(&id)
}

#[delete(Role, authz(realm = "system", skip_org))]
fn system_role_delete() {
}
