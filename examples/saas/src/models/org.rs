use crate::prelude::*;

/// A tenant organization.
#[model]
#[authz_org]
pub struct Org {
    pub name: String,
}
