use crate::prelude::*;

/// Authz check parameters for a single #[authz] macro invocation.
pub struct AuthzEnsure {
    /// Role realm required, e.g. system, org, public.
    pub realm: String,
    /// Whether the check requires the role to be scoped to the request's org.
    pub org: bool,
    /// Whether the check requires the role to be assigned to the request's user.
    pub user: bool,
}

#[async_trait]
pub trait AuthzEnsureContext<'a>
where
    Self: AuthzCacheContext<'a>,
{
    /// Verify the current operation passes its authz check, errors with the
    /// configured authz_err if no role satisfies check.
    async fn authz_ensure_in_macro(&self, check: AuthzEnsure) -> Res<()> {
        let v = self.authz_with_cache(check).await?;
        if v.is_none() {
            return Err(self.authz_err().clone());
        }
        Ok(())
    }
}

#[async_trait]
impl<'a> AuthzEnsureContext<'a> for Context<'a> {
}
