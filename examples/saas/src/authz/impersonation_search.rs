use crate::prelude::*;

#[search(Impersonation, check = authz_org)]
fn impersonation_search() {
    ctx.authz_org_search::<Impersonation>().await?
}

#[count(Impersonation, check = authz_org)]
fn impersonation_count() {
    ctx.authz_org_filter::<Impersonation>().await?
}
