use crate::prelude::*;

/// Marker trait for the consumer's role model, implemented by the #[authz_role]
/// macro, backing a default AuthzRoleImpl so the app does not have to hand write
/// the realm/org/user matching query.
pub trait AuthzRoleModel
where
    Self: AuthzImplOrgId + Send + Sync,
{
    /// Realm granted across every org, tried when no role matches the requested
    /// realm, None to never fall back.
    const FALLBACK_REALM: Option<&'static str> = None;

    /// Get column realm.
    fn col_realm() -> Self::C;

    /// Converts a row into the role match the authz engine caches.
    fn authz_role_match(m: Self::M) -> Res<AuthzRoleMatch>;

    /// Build the default AuthzRoleImpl for this entity, paired with the model
    /// holding the user to role assignments.
    fn authz_default_impl<U>() -> Box<dyn AuthzRoleImpl>
    where
        U: AuthzUserInRoleModel,
    {
        Box::new(DefaultRoleImpl::<Self, U>(PhantomData))
    }
}

/// Default AuthzRoleImpl backed by a role model R and its assignment model U.
pub struct DefaultRoleImpl<R, U>(pub(crate) PhantomData<(R, U)>);

#[async_trait]
impl<R, U> AuthzRoleImpl for DefaultRoleImpl<R, U>
where
    R: AuthzRoleModel,
    U: AuthzUserInRoleModel,
{
    async fn find_matching(
        &self,
        check: &AuthzEnsure,
        role_id: &str,
        org_id: Option<&str>,
        user_id: Option<&str>,
        db: &ConnX<'_>,
    ) -> Res<Option<AuthzRoleMatch>> {
        if let Some(m) = find_by_realm::<R, U>(&check.realm, role_id, org_id, user_id, db).await? {
            return Ok(Some(m));
        }
        // The fallback realm is never org scoped, a role in it acts on any org.
        match R::FALLBACK_REALM {
            Some(realm) if realm != check.realm => find_by_realm::<R, U>(realm, role_id, None, user_id, db).await,
            _ => Ok(None),
        }
    }
}

/// Finds the role with role_id in realm, scoped to org_id when given, and
/// assigned to user_id when given. A None org_id matches rows with no org.
async fn find_by_realm<R, U>(
    realm: &str,
    role_id: &str,
    org_id: Option<&str>,
    user_id: Option<&str>,
    db: &ConnX<'_>,
) -> Res<Option<AuthzRoleMatch>>
where
    R: AuthzRoleModel,
    U: AuthzUserInRoleModel,
{
    let mut q = R::find()
        .include_deleted(false)
        .filter_by_id(role_id)
        .filter(R::col_realm().eq(realm));

    q = if let Some(org_id) = org_id {
        q.filter(R::col_org_id().eq(org_id))
    } else {
        q.filter(R::col_org_id().is_null())
    };

    if let Some(user_id) = user_id {
        let mut sub = U::find()
            .include_deleted(false)
            .select_only()
            .column(U::col_role_id())
            .filter(U::col_user_id().eq(user_id));
        sub = if let Some(org_id) = org_id {
            sub.filter(U::col_org_id().eq(org_id))
        } else {
            sub.filter(U::col_org_id().is_null())
        };
        q = q.filter(R::col_id().in_subquery(sub.into_query()));
    }

    let Some(role) = q.one(db).await? else {
        return Ok(None);
    };
    Ok(Some(R::authz_role_match(role)?))
}
