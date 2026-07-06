use crate::prelude::*;

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

#[allow(unused_variables)]
#[async_trait]
pub trait AuthzHandlers
where
    Self: Send + Sync,
{
    async fn execute_script(&self, ctx: &Context<'_>, script: &str) -> Res<Option<JsonValue>> {
        Ok(None)
    }
}

struct DefaultHandlers;
#[async_trait]
impl AuthzHandlers for DefaultHandlers {
}

/// Org lookup callbacks, non-generic: method signatures use only primitives
/// so the trait needs no type parameter.
#[async_trait]
pub trait AuthzOrgImpl
where
    Self: Send + Sync,
{
    async fn find_by_id(&self, id: &str, tx: &DatabaseTransaction) -> Res<Option<OrgMinimal>>;
}

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
