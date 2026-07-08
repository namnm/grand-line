use crate::prelude::*;

/// Non-generic config: timeouts, keys, and non-user-model handlers.
/// Add this to your schema with `.data(AuthConfig::default())`.
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

#[allow(unused_variables)]
#[async_trait]
pub trait AuthHandlers: Send + Sync {
    async fn password_validate(&self, ctx: &Context<'_>, password: &str) -> Res<bool> {
        Ok(rand_utils::password_validate(password).is_ok())
    }

    async fn otp(&self, ctx: &Context<'_>) -> Res<String> {
        Ok(rand_utils::otp())
    }

    async fn on_otp_create(&self, ctx: &Context<'_>, otp: &AuthOtpSql, otp_raw: &str) -> Res<()> {
        Ok(())
    }
}

struct DefaultHandlers;
#[async_trait]
impl AuthHandlers for DefaultHandlers {}

/// Generic user config: callbacks that receive the user's own model type.
/// Use `let auth_user_impl = AuthUserImpl::<User>::default()` for no-op handlers,
/// or `let auth_user_impl = AuthUserImpl::<User>::new(MyHandlers)` to provide custom callbacks.
/// Add this to your schema with `.data(auth_user_impl)`.
pub struct AuthUserImpl<U>
where
    U: AuthUser,
{
    pub handlers: Arc<dyn AuthUserImplHandlers<U>>,
}

impl<U> AuthUserImpl<U>
where
    U: AuthUser,
{
    pub fn new(handlers: impl AuthUserImplHandlers<U> + 'static) -> Self {
        Self {
            handlers: Arc::new(handlers),
        }
    }
}

impl<U> Default for AuthUserImpl<U>
where
    U: AuthUser,
{
    fn default() -> Self {
        Self {
            handlers: Arc::new(DefaultUserImplHandlers(PhantomData)),
        }
    }
}

#[allow(unused_variables)]
#[async_trait]
pub trait AuthUserImplHandlers<U>
where
    U: AuthUser,
    Self: Send + Sync,
{
    async fn on_register_resolve(&self, ctx: &Context<'_>, user: &U::M, ls: &LoginSessionSql) -> Res<()> {
        Ok(())
    }

    async fn on_login_resolve(&self, ctx: &Context<'_>, user: &U::M, ls: &LoginSessionSql) -> Res<()> {
        Ok(())
    }

    async fn on_forgot_resolve(&self, ctx: &Context<'_>, user: &U::M, ls: &LoginSessionSql) -> Res<()> {
        Ok(())
    }
}

struct DefaultUserImplHandlers<U>(PhantomData<U>);
#[async_trait]
impl<U> AuthUserImplHandlers<U> for DefaultUserImplHandlers<U> where U: AuthUser {}
