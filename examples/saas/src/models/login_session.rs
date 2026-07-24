use grand_line::prelude::*;

use crate::err::SaasErr;

pub const LOGIN_SESSION_COOKIE_KEY: &str = "login_session";
pub const LOGIN_SESSION_COOKIE_EXPIRES_MS: i64 = 7 * 24 * 60 * 60 * 1000;

/// A logged-in session, identified by a bearer token or cookie carrying its id
/// plus an opaque secret checked against secret_hashed.
#[model(deleted_at = false, by_id = false)]
pub struct LoginSession {
    pub user_id: String,
    #[graphql(skip)]
    pub secret_hashed: String,
    pub ip: String,
    /// User agent in json map of request headers such as user-agent or sec-ch-ua...
    pub ua: JsonValue,
}

/// To only expose secret in some operations, not the others.
pub struct LoginSessionWithSecret {
    pub inner: LoginSessionSql,
    pub secret: String,
}
#[Object]
impl LoginSessionWithSecret {
    pub async fn secret(&self) -> String {
        self.secret.clone()
    }
    pub async fn inner(&self, ctx: &Context<'_>) -> Res<LoginSessionGql> {
        let r = self.inner.clone().into_gql(ctx).await?;
        Ok(r)
    }
}

/// Resolves the current session from the Authorization bearer header (checked
/// first) or the login_session cookie, verifying the secret and expiry.
pub async fn current_login_session(ctx: &Context<'_>) -> Res<Option<LoginSessionSql>> {
    let mut t = ctx.get_authorization_token()?;
    if t.is_empty() {
        t = ctx.get_cookie(LOGIN_SESSION_COOKIE_KEY)?.unwrap_or_default();
    }
    let Some(t) = rand_utils::qs_token_parse(&t) else {
        return Ok(None);
    };

    let tx = &*ctx.tx().await?;
    let Some(ls) = LoginSession::find().include_deleted(false).filter_by_id(&t.id).one(tx).await? else {
        return Ok(None);
    };

    if !rand_utils::secret_eq(&ls.secret_hashed, &t.secret) {
        return Ok(None);
    }
    if ls.created_at < now() - duration_ms(LOGIN_SESSION_COOKIE_EXPIRES_MS) {
        return Ok(None);
    }

    Ok(Some(ls))
}

/// Like current_login_session, but errors if there is none.
pub async fn ensure_authenticated(ctx: &Context<'_>) -> Res<LoginSessionSql> {
    current_login_session(ctx).await?.ok_or_else(|| SaasErr::Unauthenticated.into())
}

/// Errors if the caller is already authenticated (register/login/forgot guard).
pub async fn ensure_not_authenticated(ctx: &Context<'_>) -> Res<()> {
    if current_login_session(ctx).await?.is_some() {
        return Err(SaasErr::AlreadyAuthenticated.into());
    }
    Ok(())
}

pub fn set_login_session_cookie(ctx: &Context<'_>, ls: &LoginSessionWithSecret) -> Res<()> {
    let token = rand_utils::qs_token(&ls.inner.id, &ls.secret)?;
    ctx.set_cookie(LOGIN_SESSION_COOKIE_KEY, &token, LOGIN_SESSION_COOKIE_EXPIRES_MS);
    Ok(())
}

/// Creates a login session row for user_id and sets the cookie.
pub async fn login_session_create(ctx: &Context<'_>, tx: &DatabaseTransaction, user_id: &str) -> Res<LoginSessionWithSecret> {
    let ip = ctx.get_ip()?;
    let ua = ctx.get_ua()?;
    let secret = rand_utils::secret();
    let ls = am_create!(LoginSession {
        user_id: user_id.to_owned(),
        secret_hashed: rand_utils::secret_hash(&secret),
        ip,
        ua: ua.to_json()?,
    })
    .exec_without_ctx(tx)
    .await?;

    let lsws = LoginSessionWithSecret {
        inner: ls,
        secret,
    };
    set_login_session_cookie(ctx, &lsws)?;
    Ok(lsws)
}
