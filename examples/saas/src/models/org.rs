use grand_line::prelude::*;

/// A tenant organization.
#[model]
pub struct Org {
    pub name: String,
}

impl AuthzOrg for Org {
}
