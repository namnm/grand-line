use grand_line::prelude::*;

#[model]
#[authz_org]
pub struct Org {
    pub name: String,
    #[default("")]
    pub description: String,
}
