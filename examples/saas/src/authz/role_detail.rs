use crate::prelude::*;

#[detail(Role, check = authz_org)]
fn resolver() {
    ctx.authz_org_filter::<Role>().await?
}
