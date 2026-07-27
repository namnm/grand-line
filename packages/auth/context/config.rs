use crate::prelude::*;

// ---------------------------------------------------------------------------
// Auth runtime configuration
// ---------------------------------------------------------------------------

/// Runtime configuration for the auth package, cookie settings and OTP limits.
#[derive(Clone)]
pub struct AuthConfig {
    pub cookie_login_session_key: &'static str,
    pub cookie_login_session_expires_ms: i64,
    pub otp_max_attempt: i64,
    pub otp_expires_ms: i64,
    pub otp_re_request_ms: i64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            cookie_login_session_key: LOGIN_SESSION_COOKIE_KEY,
            cookie_login_session_expires_ms: LOGIN_SESSION_COOKIE_EXPIRES_MS,
            otp_max_attempt: AUTH_OTP_MAX_ATTEMPT,
            otp_expires_ms: AUTH_OTP_EXPIRE_MS,
            otp_re_request_ms: AUTH_OTP_RE_REQUEST_MS,
        }
    }
}

// ---------------------------------------------------------------------------
// Session lookup abstraction
// ---------------------------------------------------------------------------

/// Result of a session lookup: just enough to verify a request's token and
/// identify the current user and session, not the consumer model full row.
pub struct AuthImplSession {
    pub id: String,
    pub user_id: String,
    pub secret_hashed: String,
    pub created_at: DateTimeUtc,
}
/// Minimal session to cache in request context.
pub struct AuthImplSessionCached {
    pub id: String,
    pub user_id: String,
}

/// Session lookup, consumer-implemented since it queries whatever concrete
/// login-session model the consumer app defines.
#[async_trait]
pub trait AuthSessionImpl
where
    Self: Send + Sync,
{
    async fn find(&self, ctx: &Context<'_>, id: &str) -> Res<Option<AuthImplSession>>;
}

// ---------------------------------------------------------------------------
// Otp lookup abstraction
// ---------------------------------------------------------------------------

/// Result of an otp lookup, enough for the engine to verify a resolve attempt
/// or a re-request cooldown.
pub struct AuthImplOtp {
    pub id: String,
    pub email: String,
    pub secret_hashed: String,
    pub otp_salt: String,
    pub otp_hashed: String,
    pub total_attempt: i64,
    pub created_at: DateTimeUtc,
    pub data: JsonValue,
}

/// Otp lookup and mutation, consumer-implemented since it queries whatever
/// concrete otp model the consumer app defines.
#[async_trait]
pub trait AuthOtpImpl
where
    Self: Send + Sync,
{
    /// Finds the pending otp row for ty/email, used for re-request cooldown checks.
    async fn find(&self, ctx: &Context<'_>, ty: &str, email: &str) -> Res<Option<AuthImplOtp>>;
    /// Atomically increments the attempt counter for the row matching id/ty.
    async fn increment(&self, ctx: &Context<'_>, id: &str, ty: &str) -> Res<Option<AuthImplOtp>>;
    /// Resets the attempt counter to 0 after a successful resolve.
    async fn reset(&self, ctx: &Context<'_>, id: &str) -> Res<()>;
    /// Deletes the pending otp row for ty/email once its re-request cooldown has passed.
    async fn delete(&self, ctx: &Context<'_>, ty: &str, email: &str) -> Res<()>;
}
