use crate::prelude::*;

#[async_trait]
pub trait AuthContext<'a>
where
    Self: AuthConfigContext<'a> + AuthHttpContext<'a> + AuthCacheContext<'a> + AuthEnsureContext<'a>,
{
    /// Returns the authenticated user id, or MyErr::Unauthenticated if no valid session exists.
    async fn auth(&self) -> Res<String> {
        let user_id = self
            .auth_unchecked()
            .await?
            .as_ref()
            .as_ref()
            .map(|ls| ls.user_id.clone())
            .ok_or(MyErr::Unauthenticated)?;
        Ok(user_id)
    }
}

#[async_trait]
impl<'a> AuthContext<'a> for Context<'a> {
}
