use crate::prelude::*;

/// Access to the current operation's cached authz result.
#[async_trait]
pub trait AuthzRoleContext<'a>
where
    Self: AuthzCacheContext<'a>,
{
    /// Return the cached role/org for the current operation's authz check.
    /// Errors with MissingGuard if no authz guard ran on the root
    /// resolver, or with the configured authz_err if the check found no role.
    ///
    /// With more than one guard on the resolver every check is evaluated, so
    /// reaching here means all of them passed, and the role this reads, hence
    /// the row policy that follows, is the first guard's. Deliberate and
    /// positional: the guards are listed in check(..) order. Put the guard whose
    /// row policy should apply first.
    async fn authz_role(&self) -> Res<Arc<AuthzCacheItem>> {
        let k = self.authz_cache_key().await?;
        let m = self.authz_cache_or_init().await?;
        let guard = m.lock().await;
        let v = guard
            .get(&k)
            .and_then(|es| es.first())
            .ok_or(MyErr::MissingGuard)?
            .1
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
