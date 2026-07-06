#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

#[path = "./setup.rs"]
mod setup;
pub use setup::*;
#[path = "./row_handlers.rs"]
mod row_handlers;
pub use row_handlers::*;

// ---------------------------------------------------------------------------
// CRUD resolvers with authz row filtering
// authz_row defaults to true in tests via resolver_authz_row feature flag.
// ---------------------------------------------------------------------------

#[search(Task, authz(realm = "org"))]
fn task_search() {
}

#[count(Task, authz(realm = "org"))]
fn task_count() {
}

#[detail(Task, authz(realm = "org"))]
fn task_detail() {
}

#[delete(Task, authz(realm = "org"))]
fn task_delete() {
}

#[gql_input]
pub struct TaskUpdate {
    pub title: String,
}

#[update(Task, authz(realm = "org"))]
fn task_update() {
    am_update!(Task {
        id: id.clone(),
        title: data.title,
    })
}

#[derive(Default, MergedObject)]
pub struct CrudQ(TaskSearchQuery, TaskCountQuery, TaskDetailQuery);
#[derive(Default, MergedObject)]
pub struct CrudM(TaskDeleteMutation, TaskUpdateMutation);

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

pub struct RowCrudSetup {
    pub tmp: TmpDb,
    pub schema: GraphQLSchema<CrudQ, CrudM, EmptySubscription>,
    // task1: org1, task2: org2
    pub task1_id: String,
    pub task2_id: String,
}

pub async fn row_crud_setup(row_pol: RowPolicy, cfg: AuthzConfig) -> Res<RowCrudSetup> {
    let org_impl = Org::authz_default_impl();
    let tmp = tmp_db!(User, LoginSession, Org, Role, UserInRole, Task);
    let s = schema_qm::<CrudQ, CrudM>(&tmp.db).data(org_impl).data(cfg);

    let h = init_common_headers();
    let seed = seed_org_admin(&tmp, &h, "walter@example.com", row_pol).await?;

    // task1: org1, task2: org2
    let t1 = am_create!(Task {
        title: "Analyze the sample",
        assignee_id: seed.user_id.clone(),
        org_id: seed.org_id1.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let t2 = am_create!(Task {
        title: "Interview the witness",
        assignee_id: seed.user_id.clone(),
        org_id: seed.org_id2.clone(),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let headers = auth_headers(h, &seed.org_id1, &seed.token, &seed.role_id1);

    Ok(RowCrudSetup {
        schema: s.data(headers).finish(),
        tmp,
        task1_id: t1.id,
        task2_id: t2.id,
    })
}
