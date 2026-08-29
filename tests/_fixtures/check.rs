use grand_line::prelude::*;

#[async_trait]
pub trait TestCheck<'a>
where
    Self: AuthzEnsureContext<'a>,
{
    async fn authz_org(&self) -> Res<()> {
        self.authz_ensure(AuthzEnsure::realm("org")).await
    }
    async fn authz_system(&self) -> Res<()> {
        self.authz_ensure(AuthzEnsure::realm("system").skip_org()).await
    }
    /// Runs two guards with different requirements on one resolver. The second
    /// has its own realm/org/user constraints, so it must be evaluated on its
    /// own terms, never satisfied from the first one's cache entry.
    async fn authz_org_then_system(&self) -> Res<()> {
        self.authz_org().await?;
        self.authz_system().await
    }
    /// The same check twice, the second call may legitimately reuse the first
    /// one's cache entry, they ask for exactly the same thing.
    async fn authz_org_twice(&self) -> Res<()> {
        self.authz_org().await?;
        self.authz_org().await
    }
}

#[async_trait]
impl<'a> TestCheck<'a> for Context<'a> {
}
