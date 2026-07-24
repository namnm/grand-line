use grand_line::prelude::*;

use crate::models::*;

/// A system-realm role also satisfies this (see SaasRoleImpl), so system
/// admins use the same resolver, scoped to whichever org they pass via the
/// x-org-id header, instead of a separate duplicated resolver.
#[search(Role, authz(realm = "org"))]
fn role_search() {
    let org_id = ctx.authz().await?;
    Search::default().add(RoleColumn::OrgId.eq(org_id))
}

#[count(Role, authz(realm = "org"))]
fn role_count() {
    let org_id = ctx.authz().await?;
    Filter::default().add(RoleColumn::OrgId.eq(org_id))
}
