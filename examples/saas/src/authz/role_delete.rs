use crate::prelude::*;

#[mutation(check = authz_org)]
fn role_delete(id: String) -> RoleGql {
    ctx.authz_org_soft_delete::<Role>(&id).await?
}
