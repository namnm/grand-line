use crate::prelude::*;

/// HTTP-level helpers for reading and writing the login session cookie.
#[async_trait]
pub trait AuthHttpContext<'a>
where
    Self: HttpContext<'a> + AuthConfigContext<'a>,
{
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

    fn login_session_data(&self) -> Res<LoginSessionData> {
        Ok(LoginSessionData {
            ip: self.get_ip()?,
            ua: self.get_ua()?,
        })
    }
}

#[async_trait]
impl<'a> AuthHttpContext<'a> for Context<'a> {
}
