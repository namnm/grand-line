use crate::prelude::*;

#[mutation(authz(realm = "org"))]
fn role_delete(id: String) -> RoleGql {
    ctx.authz_org_soft_delete::<Role>(&id).await?
}
