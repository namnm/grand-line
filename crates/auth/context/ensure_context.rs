use crate::prelude::*;

/// Guards enforcing the current request authentication state, wired into a
/// resolver with the check attribute, e.g. #[query(check = authenticated)].
#[async_trait]
pub trait AuthEnsureContext<'a>
where
    Self: AuthCacheContext<'a>,
{
    /// Requires a session, errors with Unauthenticated when there is none.
    async fn authenticated(&self) -> Res<()> {
        if self.auth_unchecked().await?.as_ref().is_none() {
            return Err(MyErr::Unauthenticated.into());
        }
        Ok(())
    }

    /// Requires no session, errors with AlreadyAuthenticated when one exists,
    /// for resolvers such as register or login.
    async fn unauthenticated(&self) -> Res<()> {
        if self.auth_unchecked().await?.as_ref().is_some() {
            return Err(MyErr::AlreadyAuthenticated.into());
        }
        Ok(())
    }
}

#[async_trait]
impl<'a> AuthEnsureContext<'a> for Context<'a> {
}
