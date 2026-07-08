use crate::prelude::*;

#[async_trait]
pub trait AuthCacheContext {
    async fn auth_with_cache(&self) -> Res<Arc<Option<LoginSessionSql>>>;
    async fn auth_without_cache(&self) -> Res<Option<LoginSessionSql>>;
}

#[async_trait]
impl AuthCacheContext for Context<'_> {
    async fn auth_with_cache(&self) -> Res<Arc<Option<LoginSessionSql>>> {
        let arc = self.cache(|| self.auth_without_cache()).await?;
        Ok(arc)
    }

    async fn auth_without_cache(&self) -> Res<Option<LoginSessionSql>> {
        let mut token = self.get_authorization_token()?;
        if token.is_empty() {
            token = self.get_cookie_login_session()?;
        }

        let t = rand_utils::qs_token_parse(&token);
        let Some(t) = t else {
            return Ok(None);
        };

        let lsd = login_session_data(self)?;
        let tx = &*self.tx().await?;

        let ls = LoginSession::find()
            .exclude_deleted()
            .filter_by_id(&t.id)
            .one(tx)
            .await?;
        let Some(ls) = ls else {
            return Ok(None);
        };

        if !rand_utils::secret_eq(&ls.secret_hashed, &t.secret) {
            return Ok(None);
        }

        let c = self.auth_config();
        if ls.created_at < now() - duration_ms(c.cookie_login_session_expires_ms) {
            return Ok(None);
        }

        let ls = am_update!(LoginSession {
            id: ls.id.clone(),
            ip: lsd.ip,
            ua: lsd.ua.to_json()?,
        })
        .exec_without_ctx(tx)
        .await?;

        Ok(Some(ls))
    }
}
