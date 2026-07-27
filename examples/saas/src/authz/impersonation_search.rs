use crate::prelude::*;

#[search(Impersonation, authz(realm = "org"))]
fn impersonation_search() {
    ctx.authz_org_search::<Impersonation>().await?
}

#[count(Impersonation, authz(realm = "org"))]
fn impersonation_count() {
    ctx.authz_org_filter::<Impersonation>().await?
}
