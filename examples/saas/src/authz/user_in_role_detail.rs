use crate::prelude::*;

#[detail(UserInRole, check = authz_org)]
fn resolver() {
    ctx.authz_org_filter::<UserInRole>().await?
}
