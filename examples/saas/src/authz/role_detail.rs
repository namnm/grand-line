use crate::prelude::*;

#[detail(Role, authz(realm = "org"))]
fn resolver() {
    ctx.authz_org_filter::<Role>().await?
}
