use crate::prelude::*;

#[search(Role, check = authz_org)]
fn resolver() {
    ctx.authz_org_search::<Role>().await?
}

#[count(Role, check = authz_org)]
fn resolver() {
    ctx.authz_org_filter::<Role>().await?
}
