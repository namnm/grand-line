use grand_line::prelude::*;

use crate::models::*;

#[detail(Impersonation, authz(realm = "org"))]
fn impersonation_detail() {
    let org_id = ctx.authz().await?;
    Filter::default().add(ImpersonationColumn::OrgId.eq(org_id))
}
