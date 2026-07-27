use crate::prelude::*;

/// HTTP-level helpers for reading and writing the login session cookie.
pub trait AuthHttpContext<'a>
where
    Self: HttpContext<'a> + AuthConfigContext<'a>,
{
    fn get_cookie_login_session(&self) -> Res<String> {
        let c = self.auth_config();
        let v = self.get_cookie(c.cookie_login_session_key)?.unwrap_or_default();
        Ok(v)
    }

    /// Encodes id/secret into a token and sets it as the login session cookie.
    fn auth_session_set_cookie(&self, id: &str, secret: &str) -> Res<()> {
        let c = self.auth_config();
        let token = rand_utils::qs_token(id, secret)?;
        self.set_cookie(c.cookie_login_session_key, &token, c.cookie_login_session_expires_ms);
        Ok(())
    }
}

impl<'a> AuthHttpContext<'a> for Context<'a> {
}
