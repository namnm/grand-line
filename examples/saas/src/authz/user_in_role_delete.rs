use crate::prelude::*;

#[mutation(check = authz_org)]
fn user_in_role_delete(id: String) -> UserInRoleGql {
    ctx.authz_org_soft_delete::<UserInRole>(&id).await?
}
