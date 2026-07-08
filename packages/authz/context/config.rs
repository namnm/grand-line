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

    /// Called after an invited user accepts an org invitation and a UserInRole
    /// is created for them.
    async fn on_org_invitation_resolve(&self, ctx: &Context<'_>, uir: &UserInRoleSql) -> Res<()> {
        Ok(())
    }

    /// Called after an org invitation is explicitly declined.
    async fn on_org_invitation_reject(&self, ctx: &Context<'_>, otp: &OtpSql) -> Res<()> {
        Ok(())
    }

    /// Called after an admin creates an impersonation session for another user,
    /// with the id of the Impersonation record, the rest of the details (admin,
    /// user, org, reason) are queryable from that id.
    async fn on_impersonate(&self, ctx: &Context<'_>, id: &str) -> Res<()> {
        Ok(())
    }

    /// Called after an impersonation session is revoked.
    async fn on_impersonate_revoke(&self, ctx: &Context<'_>, id: &str) -> Res<()> {
        Ok(())
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
