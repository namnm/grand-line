#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

use axum::http::{HeaderMap, HeaderValue};
pub use grand_line::prelude::*;

/// Test-only login session model, backing TestSessionImpl below via a real
/// table round-trip, so cache_context.rs's expiry/secret checks are exercised
/// for real instead of stubbed out.
#[model(deleted_at = false, by_id = false)]
pub struct Session {
    pub user_id: String,
    #[graphql(skip)]
    pub secret_hashed: String,
}

pub struct TestSessionImpl;
#[async_trait]
impl AuthSessionImpl for TestSessionImpl {
    async fn find(&self, ctx: &Context<'_>, id: &str) -> Res<Option<AuthImplSession>> {
        let tx = &*ctx.tx().await?;
        let Some(s) = Session::find().include_deleted(false).filter_by_id(id).one(tx).await? else {
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

/// Test-only otp model, backing TestOtpImpl below via a real table round-trip.
#[model(updated_at = false, deleted_at = false, by_id = false)]
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

pub struct TestOtpImpl;
#[async_trait]
impl AuthOtpImpl for TestOtpImpl {
    async fn find(&self, ctx: &Context<'_>, ty: &str, email: &str) -> Res<Option<AuthImplOtp>> {
        let db = ctx.db().await?;
        let t = Otp::find()
            .include_deleted(false)
            .filter(OtpColumn::Ty.eq(ty))
            .filter(OtpColumn::Email.eq(email))
            .one(db)
            .await?;
        Ok(t.map(Into::into))
    }

    async fn increment(&self, ctx: &Context<'_>, id: &str, ty: &str) -> Res<Option<AuthImplOtp>> {
        let db = ctx.db().await?;
        let u = Otp::update_many()
            .include_deleted(false)
            .filter_by_id(id)
            .filter(OtpColumn::Ty.eq(ty))
            .set(OtpActiveModel::defaults_on_update())
            .col_expr(OtpColumn::TotalAttempt, Expr::col(OtpColumn::TotalAttempt).add(1));

        if u.exec(db).await?.rows_affected == 0 {
            return Ok(None);
        }
        let t = Otp::find().include_deleted(false).filter_by_id(id).one(db).await?;
        Ok(t.map(Into::into))
    }

    async fn reset(&self, ctx: &Context<'_>, id: &str) -> Res<()> {
        let db = ctx.db().await?;
        Otp::update_many()
            .filter_by_id(id)
            .col_expr(OtpColumn::TotalAttempt, Expr::value(0))
            .exec(db)
            .await?;
        Ok(())
    }

    async fn delete(&self, ctx: &Context<'_>, ty: &str, email: &str) -> Res<()> {
        let db = ctx.db().await?;
        Otp::delete_many()
            .filter(OtpColumn::Ty.eq(ty))
            .filter(OtpColumn::Email.eq(email))
            .exec(db)
            .await?;
        Ok(())
    }
}

impl From<OtpSql> for AuthImplOtp {
    fn from(t: OtpSql) -> Self {
        Self {
            id: t.id,
            email: t.email,
            secret_hashed: t.secret_hashed,
            otp_salt: t.otp_salt,
            otp_hashed: t.otp_hashed,
            total_attempt: t.total_attempt,
            created_at: t.created_at,
            data: t.data,
        }
    }
}

// ---------------------------------------------------------------------------
// Note model + resolvers, exercising #[auth]/IntoAmCtx/AmExecCtx end to end
// ---------------------------------------------------------------------------

#[model]
pub struct Note {
    pub title: String,
}

#[gql_input]
pub struct NoteCreate {
    pub title: String,
}

#[create(Note, auth)]
fn note_create() {
    am_create!(Note {
        title: data.title,
    })
}

#[query(auth)]
fn ping() -> bool {
    true
}

#[query(auth(unauthenticated))]
fn only_when_logged_out() -> bool {
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
pub struct Query(PingQuery, OnlyWhenLoggedOutQuery);
#[derive(Default, MergedObject)]
pub struct Mutation(NoteCreateMutation, OtpResolveTestMutation, OtpReRequestTestMutation);

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
    let session_impl: Box<dyn AuthSessionImpl> = Box::new(TestSessionImpl);
    let otp_impl: Box<dyn AuthOtpImpl> = Box::new(TestOtpImpl);

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
