#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

#[path = "./setup.rs"]
mod setup;
pub use setup::*;
#[path = "./row_handlers.rs"]
mod row_handlers;
pub use row_handlers::*;

pub struct RowSetup {
    pub tmp: TmpDb,
    pub schema: GraphQLSchema<Query, EmptyMutation, EmptySubscription>,
}

pub async fn row_setup(row_script: Option<&str>, cfg: Option<AuthzConfig>) -> Res<RowSetup> {
    row_setup_with_col("tasks", row_script, cfg).await
}

pub async fn row_setup_with_col(col_key: &str, row_script: Option<&str>, cfg: Option<AuthzConfig>) -> Res<RowSetup> {
    let wc = col_policy_field(col_policy_fields_wildcard_nested());
    let col = col_policy(col_key.to_owned(), wc.clone(), wc);
    let row = row_script
        .map(|s| row_policy("tasks".to_owned(), s.to_owned()))
        .unwrap_or_default();
    let d = setup_with_policy(col, row).await?;

    // task1: assigned to user1, belongs to org1
    am_create!(Task {
        title: "Analyze the tissue sample",
        assignee_id: d.user_id1.clone(),
        org_id: d.org_id1.clone(),
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    // task2: assigned to user2, belongs to org2
    am_create!(Task {
        title: "Investigate the pattern",
        assignee_id: d.user_id2.clone(),
        org_id: d.org_id2.clone(),
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    let h = auth_headers(d.h, &d.org_id1, &d.token1, &d.role_id1);
    let mut b = d.s;
    if let Some(c) = cfg {
        b = b.data(c);
    }
    Ok(RowSetup {
        schema: b.data(h).finish(),
        tmp: d.tmp,
    })
}
