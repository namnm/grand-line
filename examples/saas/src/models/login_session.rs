use crate::prelude::*;

/// A logged-in session, identified by a bearer token or cookie carrying its id
/// plus an opaque secret checked against secret_hashed.
#[model(deleted_at = false, by_id = false)]
#[auth_session]
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
    pub async fn id(&self) -> String {
        self.inner.id.clone()
    }
    pub async fn secret(&self) -> String {
        self.secret.clone()
    }
    pub async fn inner(&self, ctx: &Context<'_>) -> Res<LoginSessionGql> {
        let r = self.inner.clone().into_gql(ctx).await?;
        Ok(r)
    }
}

/// Creates a login session row for user_id and sets the session cookie.
pub async fn login_session_create(ctx: &Context<'_>, db: &ConnX<'_>, user_id: &str) -> Res<LoginSessionWithSecret> {
    let secret = rand_utils::secret();
    let ls = am_create!(LoginSession {
        user_id: user_id.to_owned(),
        secret_hashed: rand_utils::secret_hash(&secret),
        ip: ctx.get_ip()?,
        ua: ctx.get_ua()?.to_json()?,
    })
    .exec_without_ctx(db)
    .await?;

    ctx.auth_session_set_cookie(&ls.id, &secret)?;
    Ok(LoginSessionWithSecret {
        inner: ls,
        secret,
    })
}
