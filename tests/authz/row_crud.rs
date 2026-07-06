#[path = "./row_crud_setup.rs"]
mod row_crud_setup;
use row_crud_setup::*;

// ---------------------------------------------------------------------------
// search tests
// ---------------------------------------------------------------------------

// No row_policy entry -> all tasks returned.
#[tokio::test]
async fn search_no_policy() -> Res<()> {
    let d = row_crud_setup(RowPolicy::default(), AuthzConfig::default()).await?;

    let q = "
    query {
        taskSearch(orderBy: [TitleAsc]) {
            title
        }
    }
    ";
    let expected = value!({
        "taskSearch": [{
            "title": "Analyze the sample",
        }, {
            "title": "Interview the witness",
        }],
    });
    exec_assert(&d.schema, q, None, &expected).await;

    d.tmp.drop().await
}

// OrgHandler returns org1 filter -> only Task1 (org1) returned.
#[tokio::test]
async fn search_with_filter() -> Res<()> {
    let pol = row_policy("taskSearch".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_crud_setup(pol, cfg).await?;

    let q = "
    query {
        taskSearch(orderBy: [TitleAsc]) {
            title
        }
    }
    ";
    let expected = value!({
        "taskSearch": [{
            "title": "Analyze the sample",
        }],
    });
    exec_assert(&d.schema, q, None, &expected).await;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// count tests
// ---------------------------------------------------------------------------

// No row_policy entry -> both tasks counted.
#[tokio::test]
async fn count_no_policy() -> Res<()> {
    let d = row_crud_setup(RowPolicy::default(), AuthzConfig::default()).await?;

    let q = "
    query {
        taskCount
    }
    ";
    let expected = value!({
        "taskCount": 2,
    });
    exec_assert(&d.schema, q, None, &expected).await;

    d.tmp.drop().await
}

// OrgHandler returns org1 filter -> only Task1 counted.
#[tokio::test]
async fn count_with_filter() -> Res<()> {
    let pol = row_policy("taskCount".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_crud_setup(pol, cfg).await?;

    let q = "
    query {
        taskCount
    }
    ";
    let expected = value!({
        "taskCount": 1,
    });
    exec_assert(&d.schema, q, None, &expected).await;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// detail tests
// ---------------------------------------------------------------------------

// No row_policy entry -> task detail returned without filter.
#[tokio::test]
async fn detail_no_policy() -> Res<()> {
    let d = row_crud_setup(RowPolicy::default(), AuthzConfig::default()).await?;

    let q = "
    query($id: ID!) {
        taskDetail(id: $id) {
            title
        }
    }
    ";
    let expected = value!({
        "taskDetail": {
            "title": "Analyze the sample",
        },
    });
    exec_assert_id(&d.schema, q, &d.task1_id, &expected).await;

    d.tmp.drop().await
}

// OrgHandler returns org1 filter, task1 belongs to org1 -> returned.
#[tokio::test]
async fn detail_filter_match() -> Res<()> {
    let pol = row_policy("taskDetail".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_crud_setup(pol, cfg).await?;

    let q = "
    query($id: ID!) {
        taskDetail(id: $id) {
            title
        }
    }
    ";
    let expected = value!({
        "taskDetail": {
            "title": "Analyze the sample",
        },
    });
    exec_assert_id(&d.schema, q, &d.task1_id, &expected).await;

    d.tmp.drop().await
}

// OrgHandler returns org1 filter, task2 belongs to org2 -> null.
#[tokio::test]
async fn detail_filter_no_match() -> Res<()> {
    let pol = row_policy("taskDetail".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_crud_setup(pol, cfg).await?;

    let q = "
    query($id: ID!) {
        taskDetail(id: $id) {
            title
        }
    }
    ";
    let expected = value!({
        "taskDetail": null,
    });
    exec_assert_id(&d.schema, q, &d.task2_id, &expected).await;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// delete tests
// ---------------------------------------------------------------------------

// No row_policy entry -> delete succeeds without filter.
// gql_delete returns G::from_id so only id is populated in the response.
#[tokio::test]
async fn delete_no_policy() -> Res<()> {
    let d = row_crud_setup(RowPolicy::default(), AuthzConfig::default()).await?;

    let q = "
    mutation($id: ID!) {
        taskDelete(id: $id, permanent: true) {
            id
        }
    }
    ";
    let expected = value!({
        "taskDelete": {
            "id": d.task1_id,
        },
    });
    exec_assert_id(&d.schema, q, &d.task1_id, &expected).await;

    d.tmp.drop().await
}

// OrgHandler returns org1 filter, task1 belongs to org1 -> delete succeeds.
#[tokio::test]
async fn delete_filter_match() -> Res<()> {
    let pol = row_policy("taskDelete".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_crud_setup(pol, cfg).await?;

    let q = "
    mutation($id: ID!) {
        taskDelete(id: $id, permanent: true) {
            id
        }
    }
    ";
    let expected = value!({
        "taskDelete": {
            "id": d.task1_id,
        },
    });
    exec_assert_id(&d.schema, q, &d.task1_id, &expected).await;

    d.tmp.drop().await
}

// OrgHandler returns org1 filter, task2 belongs to org2 -> Unauthorized.
#[tokio::test]
async fn delete_filter_no_match() -> Res<()> {
    let pol = row_policy("taskDelete".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_crud_setup(pol, cfg).await?;

    let q = "
    mutation($id: ID!) {
        taskDelete(id: $id, permanent: true) {
            id
        }
    }
    ";
    exec_assert_err_id(&d.schema, q, &d.task2_id, &AuthzErr::Unauthorized).await?;

    d.tmp.drop().await
}

// unauthorized_err configured as Db404 -> filter mismatch looks like a missing row.
#[tokio::test]
async fn delete_unauthorized_err_as_db404() -> Res<()> {
    let pol = row_policy("taskDelete".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        unauthorized_err: CoreDbErr::Db404.into(),
        ..Default::default()
    };
    let d = row_crud_setup(pol, cfg).await?;

    let q = "
    mutation($id: ID!) {
        taskDelete(id: $id, permanent: true) {
            id
        }
    }
    ";
    exec_assert_err_id(&d.schema, q, &d.task2_id, &CoreDbErr::Db404).await?;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// update tests
// ---------------------------------------------------------------------------

// No row_policy entry -> update succeeds without filter.
#[tokio::test]
async fn update_no_policy() -> Res<()> {
    let d = row_crud_setup(RowPolicy::default(), AuthzConfig::default()).await?;

    let q = r#"
    mutation($id: ID!) {
        taskUpdate(id: $id, data: { title: "Updated" }) {
            id
            title
        }
    }
    "#;
    let expected = value!({
        "taskUpdate": {
            "id": d.task1_id,
            "title": "Updated",
        },
    });
    exec_assert_id(&d.schema, q, &d.task1_id, &expected).await;

    d.tmp.drop().await
}

// OrgHandler returns org1 filter, task1 belongs to org1 -> update succeeds.
#[tokio::test]
async fn update_filter_match() -> Res<()> {
    let pol = row_policy("taskUpdate".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_crud_setup(pol, cfg).await?;

    let q = r#"
    mutation($id: ID!) {
        taskUpdate(id: $id, data: { title: "Updated" }) {
            id
        }
    }
    "#;
    let expected = value!({
        "taskUpdate": {
            "id": d.task1_id,
        },
    });
    exec_assert_id(&d.schema, q, &d.task1_id, &expected).await;

    d.tmp.drop().await
}

// OrgHandler returns org1 filter, task2 belongs to org2 -> Unauthorized.
#[tokio::test]
async fn update_filter_no_match() -> Res<()> {
    let pol = row_policy("taskUpdate".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_crud_setup(pol, cfg).await?;

    let q = r#"
    mutation($id: ID!) {
        taskUpdate(id: $id, data: { title: "Updated" }) {
            id
        }
    }
    "#;
    exec_assert_err_id(&d.schema, q, &d.task2_id, &AuthzErr::Unauthorized).await?;

    d.tmp.drop().await
}

// Authorized update changes the row in the database.
#[tokio::test]
async fn update_authorized_changes_db_row() -> Res<()> {
    let pol = row_policy("taskUpdate".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_crud_setup(pol, cfg).await?;

    let q = r#"
    mutation($id: ID!) {
        taskUpdate(id: $id, data: { title: "Changed Title" }) {
            id
        }
    }
    "#;
    let expected = value!({
        "taskUpdate": {
            "id": d.task1_id,
        },
    });
    exec_assert_id(&d.schema, q, &d.task1_id, &expected).await;

    let Some(task) = Task::find_by_id(&d.task1_id).one(&d.tmp.db).await? else {
        return TestErr::expect("task1 should still exist after authorized update");
    };

    pretty_eq!(
        task.title,
        "Changed Title",
        "task title should change after authorized update",
    );

    d.tmp.drop().await
}

// Unauthorized update does not change the row in the database.
#[tokio::test]
async fn update_unauthorized_does_not_change_db_row() -> Res<()> {
    let pol = row_policy("taskUpdate".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_crud_setup(pol, cfg).await?;

    let q = r#"
    mutation($id: ID!) {
        taskUpdate(id: $id, data: { title: "ShouldNotChange" }) {
            id
        }
    }
    "#;
    exec_assert_err_id(&d.schema, q, &d.task2_id, &AuthzErr::Unauthorized).await?;

    // task2 belongs to org2, the update was rejected, so title must still be "Interview the witness".
    let Some(task) = Task::find_by_id(&d.task2_id).one(&d.tmp.db).await? else {
        return TestErr::expect("task2 should still exist after unauthorized update");
    };

    pretty_eq!(
        task.title,
        "Interview the witness",
        "task title should not change after unauthorized update",
    );

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// soft delete tests (permanent not set -> defaults to false -> soft delete)
// ---------------------------------------------------------------------------

// No row_policy entry -> soft delete succeeds without filter.
#[tokio::test]
async fn delete_soft_no_policy() -> Res<()> {
    let d = row_crud_setup(RowPolicy::default(), AuthzConfig::default()).await?;

    let q = "
    mutation($id: ID!) {
        taskDelete(id: $id) {
            id
        }
    }
    ";
    let expected = value!({
        "taskDelete": {
            "id": d.task1_id,
        },
    });
    exec_assert_id(&d.schema, q, &d.task1_id, &expected).await;

    // Row must still exist but with deleted_at set.
    let Some(task) = Task::find_by_id(&d.task1_id).one(&d.tmp.db).await? else {
        return TestErr::expect("task1 should still exist after soft delete");
    };

    pretty_eq!(
        task.deleted_at.is_some(),
        true,
        "task deleted_at should be set after soft delete",
    );

    d.tmp.drop().await
}

// OrgHandler returns org1 filter, task1 belongs to org1 -> soft delete succeeds.
#[tokio::test]
async fn delete_soft_filter_match() -> Res<()> {
    let pol = row_policy("taskDelete".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_crud_setup(pol, cfg).await?;

    let q = "
    mutation($id: ID!) {
        taskDelete(id: $id) {
            id
        }
    }
    ";
    let expected = value!({
        "taskDelete": {
            "id": d.task1_id,
        },
    });
    exec_assert_id(&d.schema, q, &d.task1_id, &expected).await;

    // Row must still exist but with deleted_at set.
    let Some(task) = Task::find_by_id(&d.task1_id).one(&d.tmp.db).await? else {
        return TestErr::expect("task1 should still exist after filter-matched soft delete");
    };

    pretty_eq!(
        task.deleted_at.is_some(),
        true,
        "task deleted_at should be set when soft delete filter matches",
    );

    d.tmp.drop().await
}

// OrgHandler returns org1 filter, task2 belongs to org2 -> Unauthorized,
// deleted_at remains None because the soft-delete UPDATE was blocked by the filter.
#[tokio::test]
async fn delete_soft_filter_no_match() -> Res<()> {
    let pol = row_policy("taskDelete".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_crud_setup(pol, cfg).await?;

    let q = "
    mutation($id: ID!) {
        taskDelete(id: $id) {
            id
        }
    }
    ";
    exec_assert_err_id(&d.schema, q, &d.task2_id, &AuthzErr::Unauthorized).await?;

    // Row must not be soft-deleted because the filter excluded it.
    let Some(task) = Task::find_by_id(&d.task2_id).one(&d.tmp.db).await? else {
        return TestErr::expect("task2 should still exist after filter-excluded soft delete");
    };

    pretty_eq!(
        task.deleted_at.is_none(),
        true,
        "task deleted_at should not be set after unauthorized soft delete",
    );

    d.tmp.drop().await
}
