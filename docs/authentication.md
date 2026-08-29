# Authentication

The `auth` feature does **not** ship `register`/`login`/`logout`/`forgot` resolvers, a `User` model, or an OTP/session model. It ships the primitives those resolvers are built from: a session lookup DI trait, an OTP lookup/attempt-limiting DI trait, default impls of both derived from your own models, session cookie/bearer parsing, and a handful of `ctx` methods. You write `User`, your login-session model, your OTP model, and every auth resolver yourself, on top of these primitives - see the [saas example](https://github.com/namnm/grand-line/blob/master/examples/saas/src/auth) for a complete, working implementation of the flow described below.

## Setup

Define your own login-session model and your own OTP model, and mark each with its macro right under `#[model]`. The macro implements the trait the framework's default DI impl reads, so there is no lookup code to write:

```rs
#[model(deleted_at = false, by_id = false)]
#[auth_session]
pub struct LoginSession {
    pub user_id: String,
    #[graphql(skip)]
    pub secret_hashed: String,
    pub ip: String,
    pub ua: JsonValue,
}

#[model(updated_at = false, deleted_at = false, by_id = false)]
#[auth_otp]
pub struct Otp {
    #[graphql(skip)]
    pub ty: String,
    pub email: String,
    #[graphql(skip)]
    pub secret_hashed: String,
    #[graphql(skip)]
    pub otp_salt: String,
    #[graphql(skip)]
    pub otp_hashed: String,
    #[graphql(skip)]
    pub data: JsonValue,
    #[default(0)]
    #[graphql(skip)]
    pub total_attempt: i64,
}
```

| Macro             | Fields it reads, on top of `id`/`created_at` from `#[model]`                      |
| ----------------- | --------------------------------------------------------------------------------- |
| `#[auth_session]` | `user_id`, `secret_hashed`                                                        |
| `#[auth_otp]`     | `ty`, `email`, `secret_hashed`, `otp_salt`, `otp_hashed`, `total_attempt`, `data` |

Any other column (`ip`, `ua`, ..) is yours; a missing required one is a compile error naming the field. Wire the derived impls onto the schema, optionally with `AuthConfig` (falls back to `AuthConfig::default()` if omitted):

```rs
GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
    .extension(GrandLineExtension)
    .data(Arc::new(db.clone()))
    .data(LoginSession::auth_default_impl())
    .data(Otp::auth_default_impl())
    .finish()
```

The default `AuthOtpImpl` runs on the plain db connection rather than the request transaction, so an attempt is still counted when the surrounding request later errors and rolls back.

### Cookie attributes and the client IP

`AuthConfig` holds the cookie key and expiry. The attributes the browser actually enforces live on `HttpConfig`, because they apply to every cookie `ctx.set_cookie()` writes, not just the session one:

```rs
let http_config = HttpConfig {
    cookie_same_site: SameSite::Lax,     // default
    cookie_path: "/",                    // default
    ip_source: HttpIpSource::SocketAddr, // default
};
```

- **`cookie_same_site`** is the main CSRF control for a cookie-authenticated API. `Lax` is the default and is right when the browser app and the API share an origin. A browser app served from a **different** origin than the API needs `SameSite::None`, which browsers only accept together with `Secure` (the framework always sets `Secure`). Leaving it implicit means the browser applies `Lax` anyway, and a cross-origin app then silently never sends the cookie - the request just looks unauthenticated, with no error explaining why.
- **`cookie_path`** defaults to `/`. Without it the browser derives a path from the request URI, so a cookie set from `/api/graphql` is scoped to `/api` and is not sent anywhere else.
- **`ip_source`** decides where `ctx.get_ip()` reads from. It never guesses across headers:

| `ip_source`                            | Reads                                                        | Use when                         |
| -------------------------------------- | ------------------------------------------------------------ | -------------------------------- |
| `HttpIpSource::SocketAddr` (default)   | the `x-socket-addr` header your handler sets from the socket | the app is reachable directly    |
| `HttpIpSource::Proxy { header, hops }` | the `hops`-th entry from the right of `header`               | a trusted proxy sets that header |

A header is only trustworthy behind a proxy that sets it. Reachable directly, or behind a proxy that appends rather than replaces, any client can pick its own address - and that value usually ends up persisted on the session as audit data. `hops` is `1` for a single proxy appending the address it saw, `2` when another trusted proxy sits behind it.

With `SocketAddr`, your HTTP handler is responsible for setting `x-socket-addr` from the real connection:

```rs
let addr = ConnectInfo::<SocketAddr>::from_request_parts(&mut parts, &state).await;
headers.insert(H_SOCKET_ADDR, HeaderValue::from_str(&addr.to_string())?);
```

`ctx.get_ua()` returns whatever user-agent headers the request carried, and an empty map when it carried none. A programmatic client sending no `User-Agent` is neither unusual nor wrong, so recording an empty one beats refusing the login over a purely informational field. Check the map yourself if your app wants it present.

`AuthSessionImpl`/`AuthOtpImpl` are mandatory - any resolver using an auth guard (or a `ctx.auth*` method) errors with `SessionImplNotFound`/`OtpImplNotFound` if they're missing from schema data.

### Writing the impls by hand

The macros are a shortcut, not a requirement. When the defaults don't fit (a session resolved from somewhere other than its own table, an OTP row you count attempts on differently), drop the macro and implement the DI trait yourself:

```rs
pub struct MySessionImpl;
#[async_trait]
impl AuthSessionImpl for MySessionImpl {
    async fn find(&self, ctx: &Context<'_>, id: &str) -> Res<Option<AuthImplSession>> {
        let db = &ctx.db().await?;
        let Some(s) = LoginSession::find()
            .include_deleted(false)
            .filter_by_id(id)
            .one(db)
            .await?
        else {
            return Ok(None);
        };
        Ok(Some(AuthImplSession {
            id: s.id,
            user_id: s.user_id,
            secret_hashed: s.secret_hashed,
            created_at: s.created_at,
        }))
    }
}

pub struct MyOtpImpl;
#[async_trait]
impl AuthOtpImpl for MyOtpImpl {
    async fn find(&self, ctx: &Context<'_>, ty: &str, email: &str) -> Res<Option<AuthImplOtp>> {
        // ...
    }
    async fn increment(&self, ctx: &Context<'_>, id: &str, ty: &str) -> Res<Option<AuthImplOtp>> {
        // ...
    }
    async fn reset(&self, ctx: &Context<'_>, id: &str) -> Res<()> {
        // ...
    }
    async fn delete(&self, ctx: &Context<'_>, ty: &str, email: &str) -> Res<()> {
        // ...
    }
}
```

```rs
let session_impl: Box<dyn AuthSessionImpl> = Box::new(MySessionImpl);
let otp_impl: Box<dyn AuthOtpImpl> = Box::new(MyOtpImpl);
```

## Building the OTP + session flow yourself

`auth_otp_ensure_re_request` / `auth_otp_ensure_resolve` implement the shared OTP mechanics (cooldown, attempt limiting, expiry, secret+code check) against whatever OTP row your `AuthOtpImpl` returns - your resolver decides what the OTP is _for_ via a `ty` string of your own choosing:

```rs
#[gql_input]
pub struct Register {
    pub email: Email,
    pub password: String,
}

#[mutation(check = unauthenticated)]
fn register(data: Register) -> MyOtpWithSecret {
    ctx.auth_otp_ensure_re_request("register", &data.email.0).await?;

    let otp = rand_utils::otp();
    let secret = rand_utils::secret();
    let (otp_salt, otp_hashed) = rand_utils::otp_hash(&otp)?;

    let t = am_create!(Otp {
        ty: "register".to_owned(),
        email: data.email.0,
        secret_hashed: rand_utils::secret_hash(&secret),
        otp_salt,
        otp_hashed,
        // ... any payload you need to carry to the resolve step, e.g. the
        // hashed password, stashed as your own JSON-encoded field
    })
    .exec_without_ctx(db)
    .await?;

    send_otp_by_email(&t.email, &otp).await?; // bring your own mailer

    MyOtpWithSecret {
        inner: t,
        secret,
    }
}

#[mutation(check = unauthenticated)]
fn register_resolve(data: OtpResolve) -> MyLoginSessionWithSecret {
    let m = ctx
        .auth_otp_ensure_resolve("register", &data.id, &data.secret, &data.otp)
        .await?;
    // create the User row from whatever you stashed on the otp row, then:
    login_session_create(ctx, db, &user_id).await? // see below
}
```

A session-creating helper, shared by `register_resolve`/`login`/`forgot_resolve`:

```rs
async fn login_session_create(ctx: &Context<'_>, db: &ConnX<'_>, user_id: &str) -> Res<MyLoginSessionWithSecret> {
    let secret = rand_utils::secret();
    let ls = am_create!(LoginSession {
        user_id: user_id.to_owned(),
        secret_hashed: rand_utils::secret_hash(&secret),
        ip: ctx.get_ip()?,
        ua: ctx.get_ua()?.to_json()?,
    })
    .exec_without_ctx(db)
    .await?;

    ctx.auth_session_set_cookie(&ls.id, &secret)?; // optional, for cookie-based clients
    Ok(MyLoginSessionWithSecret {
        inner: ls,
        secret,
    })
}
```

`login`/`logout`/`forgot`/`forgot_resolve`/session-listing queries follow the same shape - each is a plain resolver you write, wired to `ctx.auth()`/`ctx.auth_session()`/`ctx.auth_unchecked()` and your own models. The client sends the bearer token (`Authorization: Bearer {secret}`, where `{secret}` is the `id.secret`-style token returned by `login_session_create`) or relies on the cookie set above - `ctx.auth_unchecked()` checks the `Authorization` header first, falling back to the cookie if it's absent.

## `ctx` methods

| Method                                                    | Returns                        | Description                                                                  |
| --------------------------------------------------------- | ------------------------------ | ---------------------------------------------------------------------------- |
| `ctx.authenticated().await?`                              | `()`                           | Guard, errors `AuthErr::Unauthenticated` if there is no session              |
| `ctx.unauthenticated().await?`                            | `()`                           | Guard, errors `AuthErr::AlreadyAuthenticated` if a session is present        |
| `ctx.auth().await?`                                       | `String`                       | Current user's `id`, errors `AuthErr::Unauthenticated` if no session         |
| `ctx.auth_session().await?`                               | `String`                       | Current session's `id`, same error as above                                  |
| `ctx.auth_unchecked().await?`                             | `Arc<Option<AuthImplSession>>` | Current session or `None`, cached per request                                |
| `ctx.auth_otp_ensure_re_request(ty, email).await?`        | `()`                           | Enforces the re-request cooldown, deletes the stale row once it has passed   |
| `ctx.auth_otp_ensure_resolve(ty, id, secret, otp).await?` | `AuthImplOtp`                  | Validates code/secret/expiry/attempts, resets the attempt counter on success |
| `ctx.auth_session_set_cookie(id, secret)`                 | `Res<()>`                      | Sets the login session cookie (`Set-Cookie` response header)                 |
| `ctx.auth_config()`                                       | `&AuthConfig`                  | Cookie key/expiry, OTP limits (falls back to `AuthConfig::default()`)        |

## `authenticated` / `unauthenticated` guards

`auth` ships two `ctx` guards, used through the [`check`](resolvers.md#guards-check) resolver attribute:

```rs
#[query(check = authenticated)]
fn my_profile() -> UserGql {
}

#[mutation(check = unauthenticated)]
fn register(data: Register) -> MyOtpWithSecret {
}

#[search(Todo, check = authenticated)]
fn resolver() {
}
```

`authenticated` requires a session, errors `AuthErr::Unauthenticated` otherwise. `unauthenticated` requires the opposite - errors `AuthErr::AlreadyAuthenticated` if a session is already present, useful for `register`/`login` themselves.

## Errors (`AuthErr`)

| Variant                | Client-facing | Meaning                                           |
| ---------------------- | ------------- | ------------------------------------------------- |
| `Unauthenticated`      | yes           | `authenticated` (or `ctx.auth()`) with no session |
| `AlreadyAuthenticated` | yes           | `unauthenticated` with a session already present  |
| `OtpResolveInvalid`    | yes           | Wrong code/secret, expired, or over attempt limit |
| `OtpReRequestTooSoon`  | yes           | Re-requested an OTP before its cooldown elapsed   |
