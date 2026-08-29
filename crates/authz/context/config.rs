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
    /// A row policy entry whose handler declines (execute_script returns None)
    /// denies by default: "a policy is configured but nothing handled it" and
    /// "no policy for this path" are opposites on a security boundary and must
    /// not both resolve to no filter. Set this to true to restore the lenient
    /// integration mode, where an unhandled policy entry behaves like no entry.
    pub allow_unhandled_row_policy: bool,
}

impl Default for AuthzConfig {
    fn default() -> Self {
        Self {
            org_id_header_key: H_ORG_ID,
            role_id_header_key: H_ROLE_ID,
            unauthorized_err: MyErr::Unauthorized.into(),
            handlers: Arc::new(DefaultHandlers),
            allow_unhandled_row_policy: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Pluggable script execution handlers
// ---------------------------------------------------------------------------

/// Extension points for authz behavior that depends on the consumer app,
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
    async fn find_by_id(&self, id: &str, db: &ConnX<'_>) -> Res<Option<OrgMinimal>>;
}

/// Default AuthzOrgImpl backed by any model type O implementing AuthzOrg.
pub struct DefaultOrgImpl<O>(pub(crate) PhantomData<O>);
#[async_trait]
impl<O> AuthzOrgImpl for DefaultOrgImpl<O>
where
    O: AuthzOrg,
{
    async fn find_by_id(&self, id: &str, db: &ConnX<'_>) -> Res<Option<OrgMinimal>> {
        let r = O::find()
            .include_deleted(false)
            .filter_by_id(id)
            .select_only()
            .column(O::col_id())
            .into_model::<OrgMinimal>()
            .one(db)
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

/// Role/user-assignment lookup, consumer-implemented since it queries whatever
/// concrete Role/UserInRole models the consumer app defines.
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
        db: &ConnX<'_>,
    ) -> Res<Option<AuthzRoleMatch>>;
}
