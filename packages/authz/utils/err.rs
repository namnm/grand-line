use crate::prelude::*;

/// Errors surfaced by the authz package, split into client-facing and
/// server-only variants.
#[grand_line_err]
pub enum MyErr {
    // ========================================================================
    // client errors
    //
    #[error("unauthorized")]
    #[client]
    Unauthorized,
    #[error("org id is missing in the request headers")]
    #[client]
    HeaderOrgId404,
    #[error("role id is missing in the request headers")]
    #[client]
    HeaderRoleId404,

    // ========================================================================
    // server errors
    //
    #[error("authz requires macro call in the resolver definition")]
    MissingMacro,
    #[error("authz org impl not found")]
    OrgImplNotFound,
    #[error("authz row script not found in db")]
    RowScript404,
    #[error("authz row cache downcast failed")]
    RowCacheDowncast,
}
