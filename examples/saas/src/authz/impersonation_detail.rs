use crate::prelude::*;

#[detail(Impersonation, check = authz_org)]
fn impersonation_detail() {
    ctx.authz_org_filter::<Impersonation>().await?
}
