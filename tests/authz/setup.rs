#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

use axum::http::HeaderMap;
pub use grand_line::prelude::*;

pub fn auth_headers(mut h: HeaderMap, org_id: &str, token: &str, role_id: &str) -> HeaderMap {
    h.append(H_ORG_ID, h_str(org_id));
    h.insert(H_AUTHORIZATION, h_bearer(token));
    h.insert(H_ROLE_ID, h_str(role_id));
    h
}

#[path = "../_fixtures/user.rs"]
mod user;
pub use user::*;
#[path = "../_fixtures/org.rs"]
mod org;
pub use org::*;
#[path = "../_fixtures/task.rs"]
mod task;
pub use task::*;
#[path = "../_fixtures/col_policy.rs"]
mod col_policy;
pub use col_policy::*;
#[path = "../_fixtures/row_policy.rs"]
mod row_policy;
pub use row_policy::*;

#[query(authz(realm = "org"))]
fn org_primitive() -> i64 {
    0
}

#[query(authz(realm = "org"))]
fn org() -> OrgGql {
    let org_id = ctx.authz().await?;
    Org::find()
        .include_deleted(false)
        .filter_by_id(&org_id)
        .gql_select(ctx)?
        .one_or_404(tx)
        .await?
}

#[query(authz(realm = "system", skip_org))]
fn system_primitive() -> i64 {
    0
}

#[query(authz(realm = "system", skip_org))]
fn system(org_id: String) -> OrgGql {
    Org::find()
        .include_deleted(false)
        .filter_by_id(&org_id)
        .gql_select(ctx)?
        .one_or_404(tx)
        .await?
}

#[derive(Default, MergedObject)]
pub struct Query(
    TasksQuery,
    OrgPrimitiveQuery,
    OrgQuery,
    SystemPrimitiveQuery,
    SystemQuery,
);

pub struct Setup {
    pub tmp: TmpDb,
    pub s: SchemaBuilder<Query, EmptyMutation, EmptySubscription>,
    pub h: HeaderMap,
    pub user_id1: String,
    pub user_id2: String,
    pub token1: String,
    pub token2: String,
    pub org_id1: String,
    pub org_id2: String,
    pub role_id1: String,
    pub role_id1_system: String,
    pub role_id2: String,
}

pub async fn setup_with_col_wildcard() -> Res<Setup> {
    setup_with_col_policy(col_policy_wildcard()).await
}

pub async fn setup_with_col_policy(org1_admin: ColPolicy) -> Res<Setup> {
    setup_with_policy(org1_admin, RowPolicy::default()).await
}

pub async fn setup_with_policy(org1_admin: ColPolicy, org1_row: RowPolicy) -> Res<Setup> {
    let org_impl = Org::authz_default_impl();

    let tmp = tmp_db!(User, LoginSession, Org, Role, UserInRole, Task);
    let s = schema_q::<Query>(&tmp.db).data(org_impl);

    let h = init_common_headers();

    let u1 = am_create!(User {
        email: "olivia@example.com",
        password_hashed: rand_utils::password_hash("123123")?,
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let u2 = am_create!(User {
        email: "peter@example.com",
        password_hashed: rand_utils::password_hash("123123")?,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let ua = Context::get_ua_raw(Context::axum_headers(&h))?;

    let secret1 = rand_utils::secret();
    let ls1 = am_create!(LoginSession {
        user_id: u1.id.clone(),
        secret_hashed: rand_utils::secret_hash(&secret1),
        ip: "127.0.0.1",
        ua: ua.to_json()?,
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let token1 = rand_utils::qs_token(&ls1.id, &secret1)?;

    let secret2 = rand_utils::secret();
    let ls2 = am_create!(LoginSession {
        user_id: u2.id.clone(),
        secret_hashed: rand_utils::secret_hash(&secret2),
        ip: "127.0.0.1",
        ua: ua.to_json()?,
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let token2 = rand_utils::qs_token(&ls2.id, &secret2)?;

    let o1 = am_create!(Org {
        name: "Fringe",
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let o2 = am_create!(Org {
        name: "FBI",
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let r1 = am_create!(Role {
        name: "Org Admin",
        realm: "org",
        col_policy: org1_admin.to_json()?,
        row_policy: org1_row.to_json()?,
        org_id: Some(o1.id.clone()),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(UserInRole {
        user_id: u1.id.clone(),
        role_id: r1.id.clone(),
        org_id: Some(o1.id.clone()),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let r2 = am_create!(Role {
        name: "Org Admin",
        realm: "org",
        col_policy: col_policy_wildcard().to_json()?,
        row_policy: RowPolicy::default().to_json()?,
        org_id: Some(o2.id.clone()),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(UserInRole {
        user_id: u2.id.clone(),
        role_id: r2.id.clone(),
        org_id: Some(o2.id.clone()),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let r3 = am_create!(Role {
        name: "System Admin",
        realm: "system",
        col_policy: col_policy_wildcard().to_json()?,
        row_policy: RowPolicy::default().to_json()?,
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(UserInRole {
        user_id: u1.id.clone(),
        role_id: r3.id.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    Ok(Setup {
        tmp,
        s,
        h,
        user_id1: u1.id,
        user_id2: u2.id,
        token1,
        token2,
        org_id1: o1.id,
        org_id2: o2.id,
        role_id1: r1.id,
        role_id1_system: r3.id,
        role_id2: r2.id,
    })
}

pub struct OrgAdminSeed {
    pub user_id: String,
    pub token: String,
    pub org_id1: String,
    pub org_id2: String,
    pub role_id1: String,
}

pub async fn seed_org_admin(tmp: &TmpDb, h: &HeaderMap, email: &str, row_pol: RowPolicy) -> Res<OrgAdminSeed> {
    let ua = Context::get_ua_raw(Context::axum_headers(h))?;

    let u1 = am_create!(User {
        email: email.to_owned(),
        password_hashed: rand_utils::password_hash("pw")?,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let secret1 = rand_utils::secret();
    let ls1 = am_create!(LoginSession {
        user_id: u1.id.clone(),
        secret_hashed: rand_utils::secret_hash(&secret1),
        ip: "127.0.0.1",
        ua: ua.to_json()?,
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let token1 = rand_utils::qs_token(&ls1.id, &secret1)?;

    let o1 = am_create!(Org {
        name: "Fringe Division"
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let o2 = am_create!(Org {
        name: "Massive Dynamic"
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let r1 = am_create!(Role {
        name: "Admin",
        realm: "org",
        col_policy: col_policy_wildcard().to_json()?,
        row_policy: row_pol.to_json()?,
        org_id: Some(o1.id.clone()),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(UserInRole {
        user_id: u1.id.clone(),
        role_id: r1.id.clone(),
        org_id: Some(o1.id.clone()),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    Ok(OrgAdminSeed {
        user_id: u1.id,
        token: token1,
        org_id1: o1.id,
        org_id2: o2.id,
        role_id1: r1.id,
    })
}
