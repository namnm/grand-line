use crate::prelude::*;

/// Access to the current operation's cached authz result.
#[async_trait]
pub trait AuthzRoleContext<'a>
where
    Self: AuthzCacheContext<'a>,
{
    /// Return the cached role/org for the current operation's authz check.
    /// Errors with MissingMacro if #[authz] was not applied to the root
    /// resolver, or with the configured authz_err if the check found no role.
    async fn authz_role(&self) -> Res<Arc<AuthzCacheItem>> {
        let k = self.authz_cache_key().await?;
        let m = self.authz_cache_or_init().await?;
        let guard = m.lock().await;
        let v = guard
            .get(&k)
            .ok_or(MyErr::MissingMacro)?
            .as_ref()
            .ok_or_else(|| self.authz_err().clone())?;
        let v = Arc::clone(v);
        drop(guard);
        Ok(v)
    }
}

#[async_trait]
impl<'a> AuthzRoleContext<'a> for Context<'a> {
}
