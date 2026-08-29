use crate::prelude::*;

/// Provides request-scoped, cached access to the current session match.
#[async_trait]
pub trait AuthCacheContext<'a>
where
    Self: AuthHttpContext<'a>,
{
    /// Returns the current session match if any, cached for the lifetime of the request.
    async fn auth_unchecked(&self) -> Res<Arc<Option<AuthImplSessionCached>>> {
        let arc = self.cache(|| self.auth_unchecked_without_cache()).await?;
        Ok(arc)
    }

    /// Resolves the session from the request token, without using the cache.
    async fn auth_unchecked_without_cache(&self) -> Res<Option<AuthImplSessionCached>>;
}

#[async_trait]
impl<'a> AuthCacheContext<'a> for Context<'a> {
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "shifting now by a session expiry the app configures, not by client input"
    )]
    async fn auth_unchecked_without_cache(&self) -> Res<Option<AuthImplSessionCached>> {
        let mut t = self.get_authorization_token()?;
        if t.is_empty() {
            t = self.get_cookie_login_session()?;
        }

        let Some(t) = rand_utils::qs_token_parse(&t) else {
            return Ok(None);
        };

        let Some(m) = self.auth_session_impl()?.find(self, &t.id).await? else {
            return Ok(None);
        };

        if !rand_utils::secret_eq(&m.secret_hashed, &t.secret) {
            return Ok(None);
        }

        let c = self.auth_config();
        if m.created_at < now() - duration_ms(c.cookie_login_session_expires_ms) {
            return Ok(None);
        }

        let c = AuthImplSessionCached {
            id: m.id,
            user_id: m.user_id,
        };
        Ok(Some(c))
    }
}
