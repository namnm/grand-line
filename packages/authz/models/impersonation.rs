use crate::prelude::*;

/// Records an admin creating a login session on behalf of another user, for
/// remote debugging or support. created_by_id (automatic) is the admin who
/// performed the impersonation.
#[model]
pub struct Impersonation {
    pub login_session_id: String,
    pub user_id: String,
    /// Set for org-realm-created impersonations, None for system realm.
    pub org_id: Option<String>,
    pub reason: String,
}
