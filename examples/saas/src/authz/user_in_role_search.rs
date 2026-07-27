use crate::prelude::*;

#[search(UserInRole, authz(realm = "org"))]
fn resolver() {
    ctx.authz_org_search::<UserInRole>().await?
}

#[count(UserInRole, authz(realm = "org"))]
fn resolver() {
    ctx.authz_org_filter::<UserInRole>().await?
}
