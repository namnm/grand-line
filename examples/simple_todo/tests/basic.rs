use grand_line_examples_simple_todo::prelude::*;
use grand_line_examples_simple_todo::schema as build_schema;

// ---------------------------------------------------------------------------
// Setup: a fresh in-memory db seeded with the same rows as production's db(),
// isolated per test (unlike db(), which shares one fixed sqlite name).
// ---------------------------------------------------------------------------

async fn setup() -> Res<TmpDb> {
    let tmp = tmp_db!(Todo);

    am_create!(Todo {
        content: "2023 good bye",
        done: true,
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(Todo {
        content: "2023 great",
        done: true,
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(Todo {
        content: "2024 hello",
        done: false,
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(Todo {
        content: "2024 awesome",
        done: false,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    Ok(tmp)
}

// ---------------------------------------------------------------------------
// todoCreate -> todoDetail -> todoUpdate -> todoToggleDone -> todoDelete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_then_detail_round_trip() -> Res<()> {
    let tmp = setup().await?;
    let s = build_schema(&tmp.db).finish();

    let q = r#"
    mutation {
        todoCreate(data: { content: "review the pattern case files" }) {
            id
            content
            done
        }
    }
    "#;
    let res = exec_assert_ok(&s, q, None).await;
    let data = res.data.to_json()?;
    let id = data.str("/todoCreate/id").to_owned();
    pretty_eq!(
        data.str("/todoCreate/content"),
        "review the pattern case files",
        "created todo should have the given content",
    );

    let q = "
    query($id: ID!) {
        todoDetail(id: $id) {
            content
            done
        }
    }
    ";
    let expected = value!({
        "todoDetail": {
            "content": "review the pattern case files",
            "done": false,
        },
    });
    exec_assert_id(&s, q, &id, &expected).await;

    tmp.drop().await
}

#[tokio::test]
async fn update_changes_the_content() -> Res<()> {
    let tmp = setup().await?;
    let s = build_schema(&tmp.db).finish();

    let todo = am_create!(Todo {
        content: "brew coffee for walter",
        done: false,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    mutation($id: String!, $content: String!) {
        todoUpdate(id: $id, data: { content: $content }) {
            content
        }
    }
    ";
    let v = value!({
        "id": todo.id,
        "content": "brew coffee for walter, extra strong",
    });
    let expected = value!({
        "todoUpdate": {
            "content": "brew coffee for walter, extra strong",
        },
    });
    exec_assert(&s, q, Some(v), &expected).await;

    tmp.drop().await
}

#[tokio::test]
async fn toggle_done_flips_the_flag() -> Res<()> {
    let tmp = setup().await?;
    let s = build_schema(&tmp.db).finish();

    let todo = am_create!(Todo {
        content: "assemble the vacuum tube array",
        done: false,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    mutation($id: String!) {
        todoToggleDone(id: $id) {
            done
        }
    }
    ";
    let expected = value!({
        "todoToggleDone": {
            "done": true,
        },
    });
    exec_assert_id(&s, q, &todo.id, &expected).await;

    let expected = value!({
        "todoToggleDone": {
            "done": false,
        },
    });
    exec_assert_id(&s, q, &todo.id, &expected).await;

    tmp.drop().await
}

#[tokio::test]
async fn delete_excludes_the_todo_from_search() -> Res<()> {
    let tmp = setup().await?;
    let s = build_schema(&tmp.db).finish();

    let todo = am_create!(Todo {
        content: "track the observer sighting",
        done: false,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let q = "
    mutation($id: String!) {
        todoDelete(id: $id) {
            id
        }
    }
    ";
    exec_assert_id(&s, q, &todo.id, &value!({ "todoDelete": { "id": todo.id } })).await;

    let q = "
    query {
        todoSearch {
            content
        }
    }
    ";
    let res = exec_assert_ok(&s, q, None).await;
    let data = res.data.to_json()?;
    let still_present = data
        .arr("/todoSearch")
        .iter()
        .any(|t| t.str("/content") == "track the observer sighting");
    pretty_eq!(
        still_present,
        false,
        "a deleted todo should not appear in the default search"
    );

    tmp.drop().await
}

// ---------------------------------------------------------------------------
// todoSearch / todoSearch2024 / todoCount
// ---------------------------------------------------------------------------

#[tokio::test]
async fn search_returns_every_seeded_row() -> Res<()> {
    let tmp = setup().await?;
    let s = build_schema(&tmp.db).finish();

    let q = "
    query {
        todoSearch {
            content
        }
    }
    ";
    let res = exec_assert_ok(&s, q, None).await;
    let data = res.data.to_json()?;
    pretty_eq!(
        data.arr("/todoSearch").len(),
        4,
        "all 4 seeded todos should be returned"
    );

    tmp.drop().await
}

#[tokio::test]
async fn search_2024_filters_and_sorts_by_done_then_content() -> Res<()> {
    let tmp = setup().await?;
    let s = build_schema(&tmp.db).finish();

    let q = "
    query {
        todoSearch2024 {
            content
            done
        }
    }
    ";
    let expected = value!({
        "todoSearch2024": [
            { "content": "2024 awesome", "done": false },
            { "content": "2024 hello", "done": false },
        ],
    });
    exec_assert(&s, q, None, &expected).await;

    tmp.drop().await
}

#[tokio::test]
async fn count_reports_the_matching_total() -> Res<()> {
    let tmp = setup().await?;
    let s = build_schema(&tmp.db).finish();

    let q = "
    query {
        todoCount
    }
    ";
    let expected = value!({
        "todoCount": 4,
    });
    exec_assert(&s, q, None, &expected).await;

    tmp.drop().await
}

// ---------------------------------------------------------------------------
// Manual resolvers: todoCountDone / todoDeleteDone / hello
// ---------------------------------------------------------------------------

#[tokio::test]
async fn count_done_counts_only_the_done_rows() -> Res<()> {
    let tmp = setup().await?;
    let s = build_schema(&tmp.db).finish();

    let q = "
    query {
        todoCountDone
    }
    ";
    let expected = value!({
        "todoCountDone": 2,
    });
    exec_assert(&s, q, None, &expected).await;

    tmp.drop().await
}

#[tokio::test]
async fn delete_done_soft_deletes_every_done_row() -> Res<()> {
    let tmp = setup().await?;
    let s = build_schema(&tmp.db).finish();

    // todo_delete_done's resolver only gql_select_id()s the affected rows, so
    // only "id" is safe to request here, other fields are left unresolved.
    let q = "
    mutation {
        todoDeleteDone {
            id
        }
    }
    ";
    let res = exec_assert_ok(&s, q, None).await;
    let data = res.data.to_json()?;
    pretty_eq!(
        data.arr("/todoDeleteDone").len(),
        2,
        "both done todos (2023 good bye, 2023 great) should be soft deleted",
    );

    let q = "
    query {
        todoSearch {
            content
        }
    }
    ";
    let res = exec_assert_ok(&s, q, None).await;
    let data = res.data.to_json()?;
    pretty_eq!(
        data.arr("/todoSearch").len(),
        2,
        "only the 2 not-done todos should remain visible after todoDeleteDone",
    );

    tmp.drop().await
}

#[tokio::test]
async fn hello_query_returns_a_greeting() -> Res<()> {
    let tmp = setup().await?;
    let s = build_schema(&tmp.db).finish();

    let q = "
    query {
        hello {
            message
        }
    }
    ";
    let expected = value!({
        "hello": {
            "message": "hello from graphql",
        },
    });
    exec_assert(&s, q, None, &expected).await;

    tmp.drop().await
}

// ---------------------------------------------------------------------------
// REST handler: hello_rest is a plain axum handler, call it directly
// ---------------------------------------------------------------------------

#[tokio::test]
async fn hello_rest_returns_a_greeting() {
    let res = hello_rest().await;
    pretty_eq!(
        res.0.message,
        "hello",
        "the REST hello endpoint should greet with hello"
    );
}
