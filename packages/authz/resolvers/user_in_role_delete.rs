use crate::prelude::*;

#[mutation(authz(realm = "org"))]
fn org_user_in_role_delete(id: String) -> UserInRoleGql {
    let org_id = ctx.authz().await?;

    let tx = &*ctx.tx().await?;
    UserInRole::find_by_id(&id)
        .filter(UserInRoleColumn::OrgId.eq(&org_id))
        .exists_or_404(tx)
        .await?;
    UserInRole::soft_delete_by_id(&id)?
        .filter(UserInRoleColumn::OrgId.eq(&org_id))
        .exec(tx)
        .await?;

    UserInRoleGql::from_id(&id)
}
