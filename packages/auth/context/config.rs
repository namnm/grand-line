use crate::prelude::*;

// ---------------------------------------------------------------------------
// Auth configuration
// ---------------------------------------------------------------------------

/// Runtime configuration for the auth package, cookie settings, OTP limits, and pluggable handlers.
#[derive(Clone)]
pub struct AuthConfig {
    pub cookie_login_session_key: &'static str,
    pub cookie_login_session_expires_ms: i64,
    pub otp_max_attempt: i64,
    pub otp_expires_ms: i64,
    pub otp_re_request_ms: i64,
    pub handlers: Arc<dyn AuthHandlers>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            cookie_login_session_key: LOGIN_SESSION_COOKIE_KEY,
            cookie_login_session_expires_ms: LOGIN_SESSION_COOKIE_EXPIRES,
            otp_max_attempt: AUTH_OTP_MAX_ATTEMPT,
            otp_expires_ms: AUTH_OTP_EXPIRE_MS,
            otp_re_request_ms: AUTH_OTP_RE_REQUEST_MS,
            handlers: Arc::new(DefaultHandlers),
        }
    }
}

// ---------------------------------------------------------------------------
// Pluggable behavior hooks
// ---------------------------------------------------------------------------

/// Extension points for customizing auth behavior, all methods have permissive no-op defaults.
#[allow(unused_variables)]
#[async_trait]
pub trait AuthHandlers
where
    Self: Send + Sync,
{
    async fn password_validate(&self, ctx: &Context<'_>, password: &str) -> Res<bool> {
        Ok(rand_utils::password_validate(password).is_ok())
    }

    async fn otp(&self, ctx: &Context<'_>) -> Res<String> {
        Ok(rand_utils::otp())
    }

    /// Called right after an OTP row is created, with the raw (unhashed) OTP value,
    /// typically used to deliver the OTP to the user, e.g. by email.
    async fn on_otp_create(&self, ctx: &Context<'_>, otp: &AuthOtpSql, otp_raw: &str) -> Res<()> {
        Ok(())
    }

    /// Called after a new user finishes registration and a login session is created.
    async fn on_register_resolve(&self, ctx: &Context<'_>, user_id: &str, ls: &LoginSessionSql) -> Res<()> {
        Ok(())
    }

    /// Called after a user logs in and a login session is created.
    async fn on_login_resolve(&self, ctx: &Context<'_>, user_id: &str, ls: &LoginSessionSql) -> Res<()> {
        Ok(())
    }

    /// Called after a user completes the forgot-password flow and a login session is created.
    async fn on_forgot_resolve(&self, ctx: &Context<'_>, user_id: &str, ls: &LoginSessionSql) -> Res<()> {
        Ok(())
    }
}

struct DefaultHandlers;
#[async_trait]
impl AuthHandlers for DefaultHandlers {
}
