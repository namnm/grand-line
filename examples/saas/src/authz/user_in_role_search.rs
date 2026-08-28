use crate::prelude::*;

#[search(UserInRole, check = authz_org)]
fn resolver() {
    ctx.authz_org_search::<UserInRole>().await?
}

#[count(UserInRole, check = authz_org)]
fn resolver() {
    ctx.authz_org_filter::<UserInRole>().await?
}
