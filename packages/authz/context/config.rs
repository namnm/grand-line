use crate::prelude::*;

// ---------------------------------------------------------------------------
// Authz runtime configuration
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct AuthzConfig {
    pub org_id_header_key: &'static str,
    pub role_id_header_key: &'static str,
    /// Can be configured to use CoreDbErr::Db404 to not leak the existence status.
    pub unauthorized_err: GrandLineErr,
    pub handlers: Arc<dyn AuthzHandlers>,
}

impl Default for AuthzConfig {
    fn default() -> Self {
        Self {
            org_id_header_key: H_ORG_ID,
            role_id_header_key: H_ROLE_ID,
            unauthorized_err: MyErr::Unauthorized.into(),
            handlers: Arc::new(DefaultHandlers),
        }
    }
}

// ---------------------------------------------------------------------------
// Pluggable script execution handlers
// ---------------------------------------------------------------------------

/// Extension points for authz behavior that depends on the host application,
/// e.g. running row policy dsl scripts. The default implementation is a no-op.
#[allow(unused_variables)]
#[async_trait]
pub trait AuthzHandlers
where
    Self: Send + Sync,
{
    /// Execute a row policy dsl script and return the resulting json, or
    /// None if the script is not handled.
    async fn execute_script(&self, ctx: &Context<'_>, script: &str) -> Res<Option<JsonValue>> {
        Ok(None)
    }
}

struct DefaultHandlers;
#[async_trait]
impl AuthzHandlers for DefaultHandlers {
}

// ---------------------------------------------------------------------------
// Org lookup abstraction
// ---------------------------------------------------------------------------

/// Org lookup callbacks, non-generic: method signatures use only primitives
/// so the trait needs no type parameter.
#[async_trait]
pub trait AuthzOrgImpl
where
    Self: Send + Sync,
{
    async fn find_by_id(&self, id: &str, tx: &DatabaseTransaction) -> Res<Option<OrgMinimal>>;
}

/// Default AuthzOrgImpl backed by any model type O implementing AuthzOrg.
pub struct DefaultOrgImpl<O>(pub(crate) PhantomData<O>);
#[async_trait]
impl<O> AuthzOrgImpl for DefaultOrgImpl<O>
where
    O: AuthzOrg,
{
    async fn find_by_id(&self, id: &str, tx: &DatabaseTransaction) -> Res<Option<OrgMinimal>> {
        let r = O::find()
            .include_deleted(false)
            .filter_by_id(id)
            .select_only()
            .column(O::col_id())
            .into_model::<OrgMinimal>()
            .one(tx)
            .await?;
        Ok(r)
    }
}

// ---------------------------------------------------------------------------
// Role lookup abstraction
// ---------------------------------------------------------------------------

/// Result of a role lookup that satisfied an AuthzEnsure check: the role's own
/// id (for caching/row-policy lookups) plus its parsed col/row policy.
pub struct AuthzRoleMatch {
    pub role_id: String,
    pub col_policy: ColPolicy,
    pub row_policy: RowPolicy,
}

/// Role/user-assignment lookup, host-implemented since it queries whatever
/// concrete Role/UserInRole models the host app defines. Given the role id
/// from the request header plus the realm/org/user constraints of a single
/// #[authz] check, finds the matching role, or None if no role satisfies them.
///
/// The org passed to find_matching is resolved from the request header before
/// this method runs (see AuthzOrgImpl), independent of which role ends up
/// matching, so a host implementation is free to look past the realm named in
/// check.realm. A common use of this is letting a broader realm stand in for
/// a narrower one, e.g. a "system" realm role satisfying an "org" realm check
/// by retrying the lookup with realm = "system" and org_id = None when the
/// direct org-scoped lookup finds nothing. This lets one resolver written
/// against a single realm serve both a tenant-scoped actor and a cross-tenant
/// one, instead of duplicating every resolver per realm, see SaasRoleImpl in
/// examples/saas/src/authz_impl.rs for a worked example.
#[async_trait]
pub trait AuthzRoleImpl
where
    Self: Send + Sync,
{
    async fn find_matching(
        &self,
        check: &AuthzEnsure,
        role_id: &str,
        org_id: Option<&str>,
        user_id: Option<&str>,
        tx: &DatabaseTransaction,
    ) -> Res<Option<AuthzRoleMatch>>;
}

// ---------------------------------------------------------------------------
// Current user lookup abstraction
// ---------------------------------------------------------------------------

/// Resolves the current request's authenticated user id, host-implemented
/// since "how is a user authenticated" (session cookie, bearer token, ...) is
/// entirely up to the host app. Errors if there is no authenticated user.
#[async_trait]
pub trait AuthzCurrentUserImpl
where
    Self: Send + Sync,
{
    async fn current_user_id(&self, ctx: &Context<'_>) -> Res<String>;
}
