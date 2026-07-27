use crate::prelude::*;

#[detail(UserInRole, authz(realm = "org"))]
fn resolver() {
    ctx.authz_org_filter::<UserInRole>().await?
}
