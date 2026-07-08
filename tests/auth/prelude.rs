#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

use axum::http::HeaderMap;
pub use grand_line::prelude::*;

#[path = "../_fixtures/user.rs"]
mod user;
pub use user::*;

#[derive(Default, MergedObject)]
pub struct Query(AuthMergedQuery);
#[derive(Default, MergedObject)]
pub struct Mutation(AuthMergedMutation<User>);

pub struct Prepare {
    pub tmp: TmpDb,
    pub s: SchemaBuilder<Query, Mutation, EmptySubscription>,
    pub h: HeaderMap,
    pub user_id: String,
    pub token: String,
}

pub async fn prepare() -> Res<Prepare> {
    let tmp = tmp_db!(User, AuthOtp, LoginSession);
    let c = AuthConfig {
        handlers: Arc::new(MockAuthHandlers),
        ..Default::default()
    };
    let s = schema_qm::<Query, Mutation>(&tmp.db)
        .data(c)
        .data(AuthUserImpl::<User>::default());
    let h = init_common_headers();

    let u = am_create!(User {
        email: "olivia@example.com",
        password_hashed: rand_utils::password_hash("123123")?,
        display_name: "Olivia",
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let ua = Context::get_ua_raw(Context::get_headers_raw(&h))?;
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

    Ok(Prepare {
        tmp,
        s,
        h,
        user_id: u.id,
        token,
    })
}

pub struct MockAuthHandlers;
#[async_trait]
impl AuthHandlers for MockAuthHandlers {
    async fn otp(&self, _ctx: &Context<'_>) -> Res<String> {
        Ok("999999".to_owned())
    }
}
