#[path = "./row_relation_setup.rs"]
mod row_relation_setup;
use row_relation_setup::*;

// ---------------------------------------------------------------------------
// has_many tests
// ---------------------------------------------------------------------------

// No row_policy entry -> all comments returned (DataLoader path).
#[tokio::test]
async fn has_many_no_policy() -> Res<()> {
    let d = row_relation_setup(RowPolicy::default(), AuthzConfig::default()).await?;

    let q = "
    query($id: ID!) {
        postDetail(id: $id) {
            comments(orderBy: [BodyAsc]) {
                body
            }
        }
    }
    ";
    let expected = value!({
        "postDetail": {
            "comments": [{
                "body": "Anomaly detected",
            }, {
                "body": "Case closed",
            }],
        },
    });
    exec_assert_id(&d.schema, q, &d.post1_id, &expected).await;

    d.tmp.drop().await
}

// OrgHandler returns org1 filter -> only comment A (org1) returned.
#[tokio::test]
async fn has_many_with_filter() -> Res<()> {
    let pol = row_policy("postDetail.comments".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_relation_setup(pol, cfg).await?;

    let q = "
    query($id: ID!) {
        postDetail(id: $id) {
            comments {
                body
            }
        }
    }
    ";
    let expected = value!({
        "postDetail": {
            "comments": [{
                "body": "Anomaly detected",
            }],
        },
    });
    exec_assert_id(&d.schema, q, &d.post1_id, &expected).await;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// has_one tests
// ---------------------------------------------------------------------------

// No row_policy entry -> meta returned via DataLoader path.
#[tokio::test]
async fn has_one_no_policy() -> Res<()> {
    let d = row_relation_setup(RowPolicy::default(), AuthzConfig::default()).await?;

    let q = "
    query($id: ID!) {
        postDetail(id: $id) {
            meta {
                text
            }
        }
    }
    ";
    let expected = value!({
        "postDetail": {
            "meta": {
                "text": "Case file notes",
            },
        },
    });
    exec_assert_id(&d.schema, q, &d.post1_id, &expected).await;

    d.tmp.drop().await
}

// OrgHandler returns org1 filter, meta1 has org1 -> returned.
#[tokio::test]
async fn has_one_filter_match() -> Res<()> {
    let pol = row_policy("postDetail.meta".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_relation_setup(pol, cfg).await?;

    let q = "
    query($id: ID!) {
        postDetail(id: $id) {
            meta {
                text
            }
        }
    }
    ";
    let expected = value!({
        "postDetail": {
            "meta": {
                "text": "Case file notes",
            },
        },
    });
    exec_assert_id(&d.schema, q, &d.post1_id, &expected).await;

    d.tmp.drop().await
}

// OrgHandler returns org1 filter, meta2 has org2 -> None.
#[tokio::test]
async fn has_one_filter_no_match() -> Res<()> {
    let pol = row_policy("postDetail.meta".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_relation_setup(pol, cfg).await?;

    let q = "
    query($id: ID!) {
        postDetail(id: $id) {
            meta {
                text
            }
        }
    }
    ";
    let expected = value!({
        "postDetail": {
            "meta": null,
        },
    });
    exec_assert_id(&d.schema, q, &d.post2_id, &expected).await;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// belongs_to tests
// ---------------------------------------------------------------------------

// No row_policy entry -> post returned via DataLoader path.
#[tokio::test]
async fn belongs_to_no_policy() -> Res<()> {
    let d = row_relation_setup(RowPolicy::default(), AuthzConfig::default()).await?;

    let q = "
    query($id: ID!) {
        commentDetail(id: $id) {
            post {
                orgId
            }
        }
    }
    ";
    let expected = value!({
        "commentDetail": {
            "post": {
                "orgId": d.org1_id,
            },
        },
    });
    exec_assert_id(&d.schema, q, &d.comment_a_id, &expected).await;

    d.tmp.drop().await
}

// OrgHandler returns org1 filter, post1 has org1 -> returned.
#[tokio::test]
async fn belongs_to_filter_match() -> Res<()> {
    let pol = row_policy("commentDetail.post".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_relation_setup(pol, cfg).await?;

    let q = "
    query($id: ID!) {
        commentDetail(id: $id) {
            post {
                id
            }
        }
    }
    ";
    let expected = value!({
        "commentDetail": {
            "post": {
                "id": d.post1_id,
            },
        },
    });
    exec_assert_id(&d.schema, q, &d.comment_a_id, &expected).await;

    d.tmp.drop().await
}

// OrgHandler returns org1 filter, comment_on_post2 points to post2 (org2) -> None.
#[tokio::test]
async fn belongs_to_filter_no_match() -> Res<()> {
    let pol = row_policy("commentDetail.post".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_relation_setup(pol, cfg).await?;

    let q = "
    query($id: ID!) {
        commentDetail(id: $id) {
            post {
                id
            }
        }
    }
    ";
    let expected = value!({
        "commentDetail": {
            "post": null,
        },
    });
    exec_assert_id(&d.schema, q, &d.comment_on_post2_id, &expected).await;
    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// many_to_many tests
// ---------------------------------------------------------------------------

// No row_policy entry -> all tags returned.
#[tokio::test]
async fn many_to_many_no_policy() -> Res<()> {
    let d = row_relation_setup(RowPolicy::default(), AuthzConfig::default()).await?;

    let q = "
    query($id: ID!) {
        postDetail(id: $id) {
            tags(orderBy: [NameAsc]) {
                name
            }
        }
    }
    ";
    let expected = value!({
        "postDetail": {
            "tags": [{
                "name": "Anomaly",
            }, {
                "name": "Pattern",
            }],
        },
    });
    exec_assert_id(&d.schema, q, &d.post1_id, &expected).await;

    d.tmp.drop().await
}

// OrgHandler returns org1 filter -> only tag T1 (org1) returned.
#[tokio::test]
async fn many_to_many_with_filter() -> Res<()> {
    let pol = row_policy("postDetail.tags".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_relation_setup(pol, cfg).await?;

    let q = "
    query($id: ID!) {
        postDetail(id: $id) {
            tags {
                name
            }
        }
    }
    ";
    let expected = value!({
        "postDetail": {
            "tags": [{
                "name": "Anomaly",
            }],
        },
    });
    exec_assert_id(&d.schema, q, &d.post1_id, &expected).await;

    d.tmp.drop().await
}
