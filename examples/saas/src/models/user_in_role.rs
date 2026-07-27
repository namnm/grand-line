use crate::prelude::*;

/// Assigns a role to a user, optionally scoped to an org.
#[model]
pub struct UserInRole {
    pub user_id: String,
    pub role_id: String,
    pub org_id: Option<String>,
}

impl AuthzImplOrgId for UserInRole {
    fn col_org_id() -> Self::C {
        UserInRoleColumn::OrgId
    }
}
