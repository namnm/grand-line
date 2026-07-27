use crate::prelude::*;

/// Umbrella trait combining every authz context trait, the entry point used
/// by the #[authz] macro.
#[async_trait]
pub trait AuthzContext<'a>
where
    Self: AuthzConfigContext<'a>
        + AuthzCacheContext<'a>
        + AuthzHttpContext<'a>
        + AuthzEnsureContext<'a>
        + AuthzRoleContext<'a>
        + AuthzColContext<'a>
        + AuthzRowContext<'a>,
{
    /// Return the org id resolved by the current operation's authz check.
    /// Errors if #[authz] was not applied, the check failed, or the check
    /// did not require org scoping.
    async fn authz(&self) -> Res<String> {
        let k = self.authz_cache_key().await?;
        let m = self.authz_cache_or_init().await?;
        let guard = m.lock().await;
        let org_id = guard
            .get(&k)
            .ok_or(MyErr::MissingMacro)?
            .as_ref()
            .as_ref()
            .ok_or_else(|| self.authz_err().clone())?
            .org
            .as_ref()
            .ok_or_else(|| self.authz_err().clone())?
            .id
            .clone();
        drop(guard);
        Ok(org_id)
    }
}

#[async_trait]
impl<'a> AuthzContext<'a> for Context<'a> {
}
