use grand_line::prelude::*;

use crate::models::*;

/// Finds the Role matching a request's role_id/realm/org/user constraints,
/// backing packages/authz's col/row-policy engine.
///
/// A system-realm role also satisfies an org-realm check: it can act on
/// behalf of whichever org is supplied via the request header, without itself
/// being tied to one org. This means org-scoped resolvers don't need a
/// separate, duplicated "system" variant -- a system admin just supplies the
/// org they want to act on and gets treated exactly like that org's own admin.
///
/// This is the general AuthzRoleImpl::find_matching pattern of letting a
/// broader realm stand in for a narrower one, applied here to two realms,
/// "system" and "org". A host with more realm tiers (e.g. "platform" above
/// "system" above "org") can chain the same idea: try the requested realm
/// first, then retry with each broader realm in turn, still with org_id set
/// to None since a broader-realm role is not tied to any single org.
pub struct SaasRoleImpl;
#[async_trait]
impl AuthzRoleImpl for SaasRoleImpl {
    async fn find_matching(
        &self,
        check: &AuthzEnsure,
        role_id: &str,
        org_id: Option<&str>,
        user_id: Option<&str>,
        tx: &DatabaseTransaction,
    ) -> Res<Option<AuthzRoleMatch>> {
        if let Some(m) = find_by_realm(&check.realm, role_id, org_id, user_id, tx).await? {
            return Ok(Some(m));
        }
        if check.realm == "org" {
            return find_by_realm("system", role_id, None, user_id, tx).await;
        }
        Ok(None)
    }
}

async fn find_by_realm(
    realm: &str,
    role_id: &str,
    org_id: Option<&str>,
    user_id: Option<&str>,
    tx: &DatabaseTransaction,
) -> Res<Option<AuthzRoleMatch>> {
    let mut q = Role::find().include_deleted(false).filter_by_id(role_id).filter(RoleColumn::Realm.eq(realm));

    q = if let Some(org_id) = org_id {
        q.filter(RoleColumn::OrgId.eq(org_id))
    } else {
        q.filter(RoleColumn::OrgId.is_null())
    };

    if let Some(user_id) = user_id {
        let mut sub = UserInRole::find()
            .include_deleted(false)
            .select_only()
            .column(UserInRoleColumn::RoleId)
            .filter(UserInRoleColumn::UserId.eq(user_id));
        sub = if let Some(org_id) = org_id {
            sub.filter(UserInRoleColumn::OrgId.eq(org_id))
        } else {
            sub.filter(UserInRoleColumn::OrgId.is_null())
        };
        q = q.filter(RoleColumn::Id.in_subquery(sub.into_query()));
    }

    let Some(role) = q.one(tx).await? else {
        return Ok(None);
    };

    Ok(Some(AuthzRoleMatch {
        role_id: role.id,
        col_policy: ColPolicy::from_json(role.col_policy)?,
        row_policy: RowPolicy::from_json(role.row_policy)?,
    }))
}

/// Resolves the current request's user id from our own login session mechanism.
pub struct SaasCurrentUserImpl;
#[async_trait]
impl AuthzCurrentUserImpl for SaasCurrentUserImpl {
    async fn current_user_id(&self, ctx: &Context<'_>) -> Res<String> {
        Ok(ensure_authenticated(ctx).await?.user_id)
    }
}
