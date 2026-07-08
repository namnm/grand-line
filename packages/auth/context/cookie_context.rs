use crate::prelude::*;

#[async_trait]
pub trait AuthCookieContext {
    fn get_cookie_login_session(&self) -> Res<String>;
    fn set_cookie_login_session(&self, ls: &LoginSessionWithSecret) -> Res<()>;
}

#[async_trait]
impl AuthCookieContext for Context<'_> {
    fn get_cookie_login_session(&self) -> Res<String> {
        let c = self.auth_config();
        let k = c.cookie_login_session_key;
        let v = self.get_cookie(k)?.unwrap_or_default();
        Ok(v)
    }

    fn set_cookie_login_session(&self, ls: &LoginSessionWithSecret) -> Res<()> {
        let c = self.auth_config();
        let k = c.cookie_login_session_key;
        let expires = c.cookie_login_session_expires_ms;
        let token = rand_utils::qs_token(&ls.inner.id, &ls.secret)?;
        self.set_cookie(k, &token, expires);
        Ok(())
    }
}
