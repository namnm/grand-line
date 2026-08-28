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
}

#[async_trait]
impl<'a> TestCheck<'a> for Context<'a> {
}
