#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

use axum::http::HeaderMap;
pub use grand_line::prelude::*;

/// Test-only header carrying the current user's id. In this test suite there is
/// no session/login mechanism at all (that now lives in examples/saas), so
/// TestCurrentUserImpl below just reads this header directly.
pub const H_USER_ID: &str = "x-user-id";

pub fn auth_headers(mut h: HeaderMap, org_id: &str, user_id: &str, role_id: &str) -> HeaderMap {
    h.append(H_ORG_ID, h_str(org_id));
    h.insert(H_USER_ID, h_str(user_id));
    h.insert(H_ROLE_ID, h_str(role_id));
    h
}

#[grand_line_err]
pub enum TestErr {
    #[error("unauthenticated")]
    #[client]
    Unauthenticated,
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

// ---------------------------------------------------------------------------
// Test-local Role / UserInRole models (packages/authz no longer owns these,
// this is what a host app's own models look like)
// ---------------------------------------------------------------------------

#[model]
pub struct Role {
    pub name: String,
    pub realm: String,
    pub col_policy: JsonValue,
    pub row_policy: JsonValue,
    pub org_id: Option<String>,
}

#[model]
pub struct UserInRole {
    pub user_id: String,
    pub role_id: String,
    pub org_id: Option<String>,
}

// ---------------------------------------------------------------------------
// DI impls wiring the local Role/UserInRole/current-user into authz's engine
// ---------------------------------------------------------------------------

pub struct TestRoleImpl;
#[async_trait]
impl AuthzRoleImpl for TestRoleImpl {
    async fn find_matching(
        &self,
        check: &AuthzEnsure,
        role_id: &str,
        org_id: Option<&str>,
        user_id: Option<&str>,
        tx: &DatabaseTransaction,
    ) -> Res<Option<AuthzRoleMatch>> {
        let mut q = Role::find()
            .include_deleted(false)
            .filter_by_id(role_id)
            .filter(RoleColumn::Realm.eq(&check.realm));

        q = if let Some(org_id) = org_id {
            q.filter(RoleColumn::OrgId.eq(org_id))
        } else {
            q.filter(RoleColumn::OrgId.is_null())
        };

        if let Some(user_id) = user_id {
            let mut sub = UserInRole::find()
                .include_deleted(false)
                .select_only()
                .column(UserInRoleColumn::RoleId)
                .filter(UserInRoleColumn::UserId.eq(user_id));
            sub = if let Some(org_id) = org_id {
                sub.filter(UserInRoleColumn::OrgId.eq(org_id))
            } else {
                sub.filter(UserInRoleColumn::OrgId.is_null())
            };
            q = q.filter(RoleColumn::Id.in_subquery(sub.into_query()));
        }

        let Some(role) = q.one(tx).await? else {
            return Ok(None);
        };

        Ok(Some(AuthzRoleMatch {
            role_id: role.id,
            col_policy: ColPolicy::from_json(role.col_policy)?,
            row_policy: RowPolicy::from_json(role.row_policy)?,
        }))
    }
}

pub struct TestCurrentUserImpl;
#[async_trait]
impl AuthzCurrentUserImpl for TestCurrentUserImpl {
    async fn current_user_id(&self, ctx: &Context<'_>) -> Res<String> {
        let v = ctx.get_header(H_USER_ID)?.trim().to_owned();
        if v.is_empty() {
            return Err(TestErr::Unauthenticated.into());
        }
        Ok(v)
    }
}

// ---------------------------------------------------------------------------
// Query resolvers
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Base two-org fixture
// ---------------------------------------------------------------------------

pub struct Setup {
    pub tmp: TmpDb,
    pub s: SchemaBuilder<Query, EmptyMutation, EmptySubscription>,
    pub h: HeaderMap,
    pub user_id1: String,
    pub user_id2: String,
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
    let role_impl: Box<dyn AuthzRoleImpl> = Box::new(TestRoleImpl);
    let user_impl: Box<dyn AuthzCurrentUserImpl> = Box::new(TestCurrentUserImpl);

    let tmp = tmp_db!(User, Org, Role, UserInRole, Task);
    let s = schema_q::<Query>(&tmp.db).data(org_impl).data(role_impl).data(user_impl);

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
        org_id1: o1.id,
        org_id2: o2.id,
        role_id1: r1.id,
        role_id1_system: r3.id,
        role_id2: r2.id,
    })
}

// ---------------------------------------------------------------------------
// Single-org admin seed
// ---------------------------------------------------------------------------

pub struct OrgAdminSeed {
    pub user_id: String,
    pub org_id1: String,
    pub org_id2: String,
    pub role_id1: String,
}

pub async fn seed_org_admin(tmp: &TmpDb, email: &str, row_pol: RowPolicy) -> Res<OrgAdminSeed> {
    let u1 = am_create!(User {
        email: email.to_owned(),
        password_hashed: rand_utils::password_hash("pw")?,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

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
        org_id1: o1.id,
        org_id2: o2.id,
        role_id1: r1.id,
    })
}
