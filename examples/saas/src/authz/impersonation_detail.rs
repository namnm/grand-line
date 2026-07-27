use crate::prelude::*;

#[detail(Impersonation, authz(realm = "org"))]
fn impersonation_detail() {
    ctx.authz_org_filter::<Impersonation>().await?
}
