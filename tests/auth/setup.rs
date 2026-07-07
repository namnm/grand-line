#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

use axum::http::HeaderMap;
pub use grand_line::prelude::*;

#[path = "../_fixtures/user.rs"]
mod user;
pub use user::*;

// ---------------------------------------------------------------------------
// Auth GraphQL query fixtures
// ---------------------------------------------------------------------------

pub const Q_LOGIN: &str = "
mutation test($data: Login!) {
    login(data: $data) {
        inner {
            userId
        }
    }
}
";

pub const Q_REGISTER: &str = "
mutation test($data: Register!) {
    register(data: $data) {
        secret
    }
}
";

pub const Q_REGISTER_RESOLVE: &str = "
mutation test($data: AuthOtpResolve!) {
    registerResolve(data: $data) {
        inner {
            userId
        }
    }
}
";

pub const Q_FORGOT: &str = "
mutation test($data: Forgot!) {
    forgot(data: $data) {
        secret
    }
}
";

pub const Q_FORGOT_RESOLVE: &str = "
mutation test($data: AuthOtpResolve!, $password: String!) {
    forgotResolve(data: $data, password: $password) {
        inner {
            userId
        }
    }
}
";

// ---------------------------------------------------------------------------
// Query and mutation root types
// ---------------------------------------------------------------------------

#[derive(Default, MergedObject)]
pub struct Query(AuthMergedQuery);
#[derive(Default, MergedObject)]
pub struct Mutation(AuthMergedMutation<User>);

// ---------------------------------------------------------------------------
// Test fixture setup
// ---------------------------------------------------------------------------

pub struct Setup {
    pub tmp: TmpDb,
    pub s: SchemaBuilder<Query, Mutation, EmptySubscription>,
    pub h: HeaderMap,
    pub user_id: String,
    pub token: String,
}

pub async fn setup() -> Res<Setup> {
    let c = AuthConfig {
        handlers: Arc::new(MockAuthHandlers),
        ..Default::default()
    };

    let tmp = tmp_db!(User, AuthOtp, LoginSession);
    let s = schema_qm::<Query, Mutation>(&tmp.db).data(c);

    let h = init_common_headers();

    let u = am_create!(User {
        email: "olivia@example.com",
        password_hashed: rand_utils::password_hash("123123")?,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let ua = Context::get_ua_raw(Context::axum_headers(&h))?;
    let secret = rand_utils::secret();
    let ls = am_create!(LoginSession {
        user_id: u.id.clone(),
        secret_hashed: rand_utils::secret_hash(&secret),
        ip: "127.0.0.1",
        ua: ua.to_json()?,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let token = rand_utils::qs_token(&ls.id, &secret)?;

    Ok(Setup {
        tmp,
        s,
        h,
        user_id: u.id,
        token,
    })
}

// ---------------------------------------------------------------------------
// Mock auth handlers
// ---------------------------------------------------------------------------

pub struct MockAuthHandlers;
#[async_trait]
impl AuthHandlers for MockAuthHandlers {
    async fn otp(&self, _ctx: &Context<'_>) -> Res<String> {
        Ok("999999".to_owned())
    }
}
