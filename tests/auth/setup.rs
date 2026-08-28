#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

use axum::http::{HeaderMap, HeaderValue};
pub use grand_line::prelude::*;

/// Test-only login session model, the framework default AuthSessionImpl is
/// derived from it by #[auth_session] so cache_context.rs's expiry/secret
/// checks are exercised against a real table round-trip.
#[model(deleted_at = false, by_id = false)]
#[auth_session]
pub struct Session {
    pub user_id: String,
    #[graphql(skip)]
    pub secret_hashed: String,
}

/// Test-only otp model, the framework default AuthOtpImpl is derived from it
/// by #[auth_otp], same real table round-trip.
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

// ---------------------------------------------------------------------------
// Note model + resolvers, exercising check/IntoAmCtx/AmExecCtx end to end
// ---------------------------------------------------------------------------

#[model]
pub struct Note {
    pub title: String,
}

#[gql_input]
pub struct NoteCreate {
    pub title: String,
}

#[create(Note, check = authenticated)]
fn note_create() {
    am_create!(Note {
        title: data.title,
    })
}

#[query(check = authenticated)]
fn ping() -> bool {
    true
}

#[query(check = unauthenticated)]
fn only_when_logged_out() -> bool {
    true
}

// ---------------------------------------------------------------------------
// Plain wrappers around the HttpContext helpers, so ip/ua/cookie behaviour can
// be exercised through a real request rather than a hand built context
// ---------------------------------------------------------------------------

#[query]
fn current_ip() -> String {
    ctx.get_ip()?
}

#[query]
fn current_ua() -> String {
    ctx.get_ua()?.to_json()?.to_string()
}

#[mutation]
fn cookie_test() -> bool {
    ctx.set_cookie("session", "walternate", 60_000);
    true
}

// ---------------------------------------------------------------------------
// Plain wrappers around ctx.auth_otp_ensure_resolve/re_request, otp mechanics
// have no macro attribute of their own so exercising them needs a resolver.
// ---------------------------------------------------------------------------

#[mutation]
async fn otp_resolve_test(ty: String, id: String, secret: String, otp: String) -> String {
    let m = ctx.auth_otp_ensure_resolve(&ty, &id, &secret, &otp).await?;
    m.data.str("/marker").to_owned()
}

#[mutation]
async fn otp_re_request_test(ty: String, email: String) -> bool {
    ctx.auth_otp_ensure_re_request(&ty, &email).await?;
    true
}

#[derive(Default, MergedObject)]
pub struct Query(PingQuery, OnlyWhenLoggedOutQuery, CurrentIpQuery, CurrentUaQuery);
#[derive(Default, MergedObject)]
pub struct Mutation(
    NoteCreateMutation,
    OtpResolveTestMutation,
    OtpReRequestTestMutation,
    CookieTestMutation,
);

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

pub const TEST_SECRET: &str = "test-secret";

pub fn h_bearer_for(id: &str, secret: &str) -> HeaderValue {
    let token = rand_utils::qs_token(id, secret).unwrap_or_default();
    h_bearer(&token)
}

pub struct Setup {
    pub tmp: TmpDb,
    pub s: SchemaBuilder<Query, Mutation, EmptySubscription>,
    pub h: HeaderMap,
}

pub async fn setup() -> Res<Setup> {
    let session_impl = Session::auth_default_impl();
    let otp_impl = Otp::auth_default_impl();

    let tmp = tmp_db!(Session, Otp, Note);
    let s = schema_qm::<Query, Mutation>(&tmp.db).data(session_impl).data(otp_impl);

    let h = init_common_headers();

    Ok(Setup {
        tmp,
        s,
        h,
    })
}

/// Creates a session row for user_id and returns the Authorization bearer
/// header value authenticating as that session.
pub async fn create_session(tmp: &TmpDb, user_id: &str) -> Res<HeaderValue> {
    let secret = rand_utils::secret();
    let s = am_create!(Session {
        user_id: user_id.to_owned(),
        secret_hashed: rand_utils::secret_hash(&secret),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    Ok(h_bearer_for(&s.id, &secret))
}
