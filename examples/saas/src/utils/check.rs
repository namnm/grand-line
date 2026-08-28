use crate::prelude::*;

/// App level guards behind the check resolver attribute, one per realm so a
/// resolver only has to name the realm it needs, e.g. #[search(Role, check = authz_org)].
#[async_trait]
pub trait SaasCheck<'a>
where
    Self: AuthzEnsureContext<'a>,
{
    /// Requires an org realm role scoped to the request org and user.
    async fn authz_org(&self) -> Res<()> {
        self.authz_ensure(AuthzEnsure::realm("org")).await
    }

    /// Requires a system realm role, not scoped to any org.
    async fn authz_system(&self) -> Res<()> {
        self.authz_ensure(AuthzEnsure::realm("system").skip_org()).await
    }
}

#[async_trait]
impl<'a> SaasCheck<'a> for Context<'a> {
}
