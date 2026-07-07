#[path = "./row_setup.rs"]
mod row_setup;
use row_setup::*;

const Q: &str = "
query {
    tasks(orderBy: [TitleAsc]) {
        title
    }
}
";

// ---------------------------------------------------------------------------
// Baseline: no filter applied
// ---------------------------------------------------------------------------

// No row_policy entry for this resolver -> all tasks returned.
#[tokio::test]
async fn no_row_policy_returns_all() -> Res<()> {
    let d = row_setup(None, None).await?;

    let expected = value!({
        "tasks": [{
            "title": "Analyze the tissue sample",
        }, {
            "title": "Investigate the pattern",
        }]
    });
    exec_assert(&d.schema, Q, None, &expected).await;

    d.tmp.drop().await
}

// execute_script returning Ok(None) -> no filter applied -> all tasks returned.
#[tokio::test]
async fn script_none_returns_all() -> Res<()> {
    let c = AuthzConfig {
        handlers: Arc::new(NoneHandler),
        ..Default::default()
    };
    let d = row_setup(Some("any"), Some(c)).await?;

    let expected = value!({
        "tasks": [{
            "title": "Analyze the tissue sample",
        }, {
            "title": "Investigate the pattern",
        }],
    });
    exec_assert(&d.schema, Q, None, &expected).await;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// Filter handlers by field source
// ---------------------------------------------------------------------------

// Handler reads ctx.auth() to get the current user, filters tasks by assignee.
#[tokio::test]
async fn script_filters_tasks_by_assignee() -> Res<()> {
    let c = AuthzConfig {
        handlers: Arc::new(AssigneeHandler),
        ..Default::default()
    };
    let d = row_setup(Some("any"), Some(c)).await?;

    // user1 is logged in, so only user1's task is returned.
    let expected = value!({
        "tasks": [{
            "title": "Analyze the tissue sample",
        }],
    });
    exec_assert(&d.schema, Q, None, &expected).await;

    d.tmp.drop().await
}

// Handler reads ctx.authz() to get the current org, filters tasks by org.
#[tokio::test]
async fn script_filters_tasks_by_org() -> Res<()> {
    let c = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_setup(Some("any"), Some(c)).await?;

    // org1 is the request context, only task belonging to org1 is returned.
    let expected = value!({
        "tasks": [{
            "title": "Analyze the tissue sample",
        }],
    });
    exec_assert(&d.schema, Q, None, &expected).await;

    d.tmp.drop().await
}

// Handler reads both user and org from ctx, filters by both assignee and org.
// row_setup's own two tasks (user1/org1, user2/org2) cannot tell a real assignee
// AND org filter apart from either field applied alone, since both single-field
// filters already narrow down to just task1. This test builds its own fixture
// with a third task that shares task1's assignee but not its org, so only a
// correct AND of both conditions excludes it.
#[tokio::test]
async fn script_filters_tasks_by_assignee_and_org() -> Res<()> {
    let wc = col_policy_field(col_policy_fields_wildcard_nested());
    let col = col_policy("tasks".to_owned(), wc.clone(), wc);
    let row = row_policy("tasks".to_owned(), "any".to_owned());
    let d = setup_with_policy(col, row).await?;

    am_create!(Task {
        title: "Analyze the tissue sample",
        assignee_id: d.user_id1.clone(),
        org_id: d.org_id1.clone(),
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;
    am_create!(Task {
        title: "Investigate the pattern",
        assignee_id: d.user_id2.clone(),
        org_id: d.org_id2.clone(),
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;
    // Same assignee as task1, different org - an assignee-only (or org-only)
    // filter would wrongly include this, only a real AND excludes it.
    am_create!(Task {
        title: "Cross reference the Cortexiphan files",
        assignee_id: d.user_id1.clone(),
        org_id: d.org_id2.clone(),
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    let c = AuthzConfig {
        handlers: Arc::new(BothHandler),
        ..Default::default()
    };
    let h = auth_headers(d.h, &d.org_id1, &d.token1, &d.role_id1);
    let schema = d.s.data(c).data(h).finish();

    let expected = value!({
        "tasks": [{
            "title": "Analyze the tissue sample",
        }],
    });
    exec_assert(&schema, Q, None, &expected).await;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// Script string forwarding
// ---------------------------------------------------------------------------

// The script string stored in row_policy is forwarded verbatim to execute_script.
#[tokio::test]
async fn script_string_forwarded_verbatim() -> Res<()> {
    let c = AuthzConfig {
        handlers: Arc::new(ScriptCheckHandler),
        ..Default::default()
    };
    let d = row_setup(Some(SCRIPT_ALPHA), Some(c)).await?;

    let expected = value!({
        "tasks": [{
            "title": "Analyze the tissue sample",
        }],
    });
    exec_assert(&d.schema, Q, None, &expected).await;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// Handler error and malformed responses
// ---------------------------------------------------------------------------

// An error from execute_script is masked as InternalServer in the GQL response.
#[tokio::test]
async fn script_error_masked_as_internal_server() -> Res<()> {
    let c = AuthzConfig {
        handlers: Arc::new(ErrorHandler),
        ..Default::default()
    };
    let d = row_setup(Some("any"), Some(c)).await?;

    exec_assert_err(&d.schema, Q, None, &CoreGraphQLErr::InternalServer).await?;

    d.tmp.drop().await
}

// Handler returns JSON with the wrong type for a filter field.
// org_id expects a String value but the handler provides a number (123).
// serde_json::from_value fails -> error propagated as InternalServer.
#[tokio::test]
async fn handler_wrong_type_returns_internal_server() -> Res<()> {
    let c = AuthzConfig {
        handlers: Arc::new(WrongTypeHandler),
        ..Default::default()
    };
    let d = row_setup(Some("any"), Some(c)).await?;

    exec_assert_err(&d.schema, Q, None, &CoreGraphQLErr::InternalServer).await?;

    d.tmp.drop().await
}

// Handler returns JSON with a field that does not exist in TaskFilter.
// TaskFilter uses #[serde(default)] without deny_unknown_fields, so the unknown
// field is silently dropped and the resulting filter is empty (all fields None).
// An empty filter applies no WHERE clause, so all tasks are returned.
#[tokio::test]
async fn handler_unknown_field_silently_ignored_returns_all() -> Res<()> {
    let c = AuthzConfig {
        handlers: Arc::new(UnknownFieldHandler),
        ..Default::default()
    };
    let d = row_setup(Some("any"), Some(c)).await?;

    let expected = value!({
        "tasks": [{
            "title": "Analyze the tissue sample",
        }, {
            "title": "Investigate the pattern",
        }],
    });
    exec_assert(&d.schema, Q, None, &expected).await;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// Wildcard column policy key
// ---------------------------------------------------------------------------

// Col policy with wildcard key "*" still applies the row filter correctly.
#[tokio::test]
async fn wildcard_col_key_with_row_filter() -> Res<()> {
    let c = AuthzConfig {
        handlers: Arc::new(AssigneeHandler),
        ..Default::default()
    };
    let d = row_setup_with_col("*", Some("any"), Some(c)).await?;

    let expected = value!({
        "tasks": [{
            "title": "Analyze the tissue sample",
        }],
    });
    exec_assert(&d.schema, Q, None, &expected).await;

    d.tmp.drop().await
}
