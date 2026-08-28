use crate::prelude::*;

/// Umbrella trait combining every auth context trait, the entry point used
/// by the authenticated/unauthenticated guards and by other crates (e.g. authz) that just need
/// to know who the current user is.
#[async_trait]
pub trait AuthContext<'a>
where
    Self:
        AuthConfigContext<'a> + AuthHttpContext<'a> + AuthCacheContext<'a> + AuthEnsureContext<'a> + AuthOtpContext<'a>,
{
    /// Returns the current authenticated user id, or MyErr::Unauthenticated if none exists.
    async fn auth(&self) -> Res<String> {
        let user_id = self
            .auth_unchecked()
            .await?
            .as_ref()
            .as_ref()
            .ok_or(MyErr::Unauthenticated)?
            .user_id
            .clone();
        Ok(user_id)
    }

    /// Returns the current authenticated session id, or MyErr::Unauthenticated if none exists.
    async fn auth_session(&self) -> Res<String> {
        let id = self
            .auth_unchecked()
            .await?
            .as_ref()
            .as_ref()
            .ok_or(MyErr::Unauthenticated)?
            .id
            .clone();
        Ok(id)
    }
}

#[async_trait]
impl<'a> AuthContext<'a> for Context<'a> {
}
