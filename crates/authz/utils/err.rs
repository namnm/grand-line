use crate::prelude::*;

/// Errors surfaced by the authz package, split into client-facing and server-only variants.
#[grand_line_err]
pub enum MyErr {
    // ------------------------------------------------------------------------
    // client errors
    // ------------------------------------------------------------------------
    #[error("unauthorized")]
    #[client]
    Unauthorized,
    #[error("org id is missing in the request headers")]
    #[client]
    HeaderOrgId404,
    #[error("role id is missing in the request headers")]
    #[client]
    HeaderRoleId404,

    // ------------------------------------------------------------------------
    // server errors
    // ------------------------------------------------------------------------
    #[error("authz requires a check guard in the resolver definition")]
    MissingGuard,
    #[error("authz org impl not found")]
    OrgImplNotFound,
    #[error("authz role impl not found")]
    RoleImplNotFound,
    #[error("authz row cache downcast failed")]
    RowCacheDowncast,
    #[error("a row policy is configured for this path but its handler did not produce a filter")]
    RowPolicyUnhandled,
    #[error(
        "row policy produced an empty filter, which would match every row, return at least one field or remove the policy entry"
    )]
    RowPolicyFilterEmpty,
    #[error("row policy filter key {k} is not a field of the target filter")]
    RowPolicyFilterKey {
        k: String,
    },
}
