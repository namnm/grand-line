use crate::prelude::*;

/// Join table assigning a role to a user, optionally scoped to an org.
#[model]
pub struct UserInRole {
    pub user_id: String,
    pub role_id: String,
    /// Can be none if this role is not related to an org. For example system admin or built in roles.
    /// If it is present, it must be matched with the associated role org.
    pub org_id: Option<String>,
}
