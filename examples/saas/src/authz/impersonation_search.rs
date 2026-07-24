use grand_line::prelude::*;

use crate::models::*;

#[search(Impersonation, authz(realm = "org"))]
fn impersonation_search() {
    let org_id = ctx.authz().await?;
    Search::default().add(ImpersonationColumn::OrgId.eq(org_id))
}

#[count(Impersonation, authz(realm = "org"))]
fn impersonation_count() {
    let org_id = ctx.authz().await?;
    Filter::default().add(ImpersonationColumn::OrgId.eq(org_id))
}
