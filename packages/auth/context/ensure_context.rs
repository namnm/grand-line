use crate::prelude::*;

/// Which authentication state auth_ensure_in_macro should check for.
pub enum AuthEnsure {
    Authenticated,
    Unauthenticated,
}

/// Guards that enforce the current request's authentication state.
#[async_trait]
pub trait AuthEnsureContext<'a>
where
    Self: AuthCacheContext<'a>,
{
    /// Dispatches to the matching ensure check, called by the auth resolver macros.
    async fn auth_ensure_in_macro(&self, check: AuthEnsure) -> Res<()> {
        match check {
            AuthEnsure::Authenticated => self.auth_ensure_authenticated().await?,
            AuthEnsure::Unauthenticated => self.auth_ensure_not_authenticated().await?,
        }
        Ok(())
    }

    async fn auth_ensure_authenticated(&self) -> Res<()> {
        if self.auth_unchecked().await?.as_ref().is_none() {
            return Err(MyErr::Unauthenticated.into());
        }
        Ok(())
    }

    async fn auth_ensure_not_authenticated(&self) -> Res<()> {
        if self.auth_unchecked().await?.as_ref().is_some() {
            return Err(MyErr::AlreadyAuthenticated.into());
        }
        Ok(())
    }
}

#[async_trait]
impl<'a> AuthEnsureContext<'a> for Context<'a> {
}
