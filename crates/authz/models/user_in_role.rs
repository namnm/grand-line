use crate::prelude::*;

/// Marker trait for the consumer's user to role assignment model, implemented by
/// the #[authz_user_in_role] macro, paired with AuthzRoleModel by DefaultRoleImpl.
pub trait AuthzUserInRoleModel
where
    Self: AuthzImplOrgId + Send + Sync,
{
    /// Get column role_id.
    fn col_role_id() -> Self::C;
    /// Get column user_id.
    fn col_user_id() -> Self::C;
}
