use crate::prelude::*;

#[search(Role, authz(realm = "org"))]
fn resolver() {
    ctx.authz_org_search::<Role>().await?
}

#[count(Role, authz(realm = "org"))]
fn resolver() {
    ctx.authz_org_filter::<Role>().await?
}
