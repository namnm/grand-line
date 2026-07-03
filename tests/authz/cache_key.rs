#[path = "./row_relation_setup.rs"]
mod row_relation_setup;
use row_relation_setup::*;

fn finish(d: Setup) -> (GraphQLSchema<Query, EmptyMutation, EmptySubscription>, TmpDb) {
    let h = auth_headers(d.h, &d.org_id1, &d.token1, &d.role_id1);
    (d.s.data(h).finish(), d.tmp)
}

// Alias: response key uses the alias, authz cache key resolves from path correctly.
#[tokio::test]
async fn alias_ok() -> Res<()> {
    let c = col_policy_with_children("org", "**");
    let (s, tmp) = finish(setup_with_col_policy(c).await?);

    let q = "
    query {
        myOrg: org {
            name
        }
    }
    ";
    let expected = value!({
        "myOrg": {
            "name": "Fringe",
        },
    });
    exec_assert(&s, q, None, &expected).await;

    tmp.drop().await
}

// Two root operations in one request each get their own independent cache entry.
#[tokio::test]
async fn alias_attack() -> Res<()> {
    let c = col_policy_with_children("orgPrimitive", "**");
    let (s, tmp) = finish(setup_with_col_policy(c).await?);

    let q = "
    query {
        orgPrimitive: org {
            name
        }
    }
    ";
    exec_assert_err(&s, q, None, &AuthzErr::Unauthorized).await;

    tmp.drop().await
}

// Two aliases for the same operation: each alias is cached independently.
#[tokio::test]
async fn two_aliases_ok() -> Res<()> {
    let c = col_policy_with_children("org", "**");
    let (s, tmp) = finish(setup_with_col_policy(c).await?);

    let q = "
    query {
        o1: org {
            name
        }
        o2: org {
            name
        }
    }
    ";
    let expected = value!({
        "o1": {
            "name": "Fringe",
        },
        "o2": {
            "name": "Fringe",
        },
    });
    exec_assert(&s, q, None, &expected).await;

    tmp.drop().await
}

// Mix of aliased and plain operations in one request.
#[tokio::test]
async fn mix_alias_and_plain() -> Res<()> {
    let mut c = col_policy_with_children("org", "**");
    c.insert(
        "orgPrimitive".to_owned(),
        col_policy_operation(
            col_policy_field(col_policy_fields_wildcard_nested()),
            col_policy_field_no_children(),
        ),
    );
    let (s, tmp) = finish(setup_with_col_policy(c).await?);

    let q = "
    query {
        myOrg: org {
            name
        }
        orgPrimitive
    }
    ";
    let expected = value!({
        "myOrg": {
            "name": "Fringe",
        },
        "orgPrimitive": 0,
    });
    exec_assert(&s, q, None, &expected).await;

    tmp.drop().await
}

// Policy allows only "org". The second operation "orgPrimitive" gets its own
// independent authz check and fails without affecting the first.
#[tokio::test]
async fn multiple_ops_one_err() -> Res<()> {
    let c = col_policy_with_children("org", "**");
    let (s, tmp) = finish(setup_with_col_policy(c).await?);

    let q = "
    query {
        org {
            name
        }
        orgPrimitive
    }
    ";
    let v = value!({
        "org": {
            "name": "Fringe",
        },
    });
    let res = s.execute(q).await;

    assert_eq!(res.errors.len(), 1, "{:#?}", res.errors);
    check_err(&res, &AuthzErr::Unauthorized);
    // async-graphql omits errored fields from data rather than setting them null.
    pretty_eq!(res.data, v);

    tmp.drop().await
}

// Aliased op succeeds, other op in same request is Unauthorized independently.
#[tokio::test]
async fn alias_ok_other_err() -> Res<()> {
    let c = col_policy_with_children("org", "**");
    let (s, tmp) = finish(setup_with_col_policy(c).await?);

    let q = "
    query {
        myOrg: org {
            name
        }
        orgPrimitive
    }
    ";
    let v = value!({
        "myOrg": {
            "name": "Fringe",
        },
    });
    let res = s.execute(q).await;

    assert_eq!(res.errors.len(), 1, "{:#?}", res.errors);
    check_err(&res, &AuthzErr::Unauthorized);
    // async-graphql omits errored fields from data rather than setting them null.
    pretty_eq!(res.data, v);

    tmp.drop().await
}

// ---------------------------------------------------------------------------
// Nested relationship walk-up tests
// ---------------------------------------------------------------------------

// Alias on root: authz_cache_key walk-up finds "pd" from the comments path
// ["pd", "comments"]. Row policy lookup translates the alias path "pd.comments"
// back to the schema field name "postDetail.comments" via the alias map, so
// the filter still applies. Aliasing cannot be used to bypass row policy.
#[tokio::test]
async fn nested_alias_on_root() -> Res<()> {
    let pol = row_policy("postDetail.comments".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_relation_setup(pol, cfg).await?;

    let q = "
    query($id: ID!) {
        pd: postDetail(id: $id) {
            comments(orderBy: [BodyAsc]) {
                body
            }
        }
    }
    ";
    let expected = value!({
        "pd": {
            "comments": [{
                "body": "Anomaly detected",
            }],
        },
    });
    exec_assert_id(&d.schema, q, &d.post1_id, &expected).await;

    d.tmp.drop().await
}

// Both the root field and a nested field carry aliases:
// pd: postDetail { cmt: comments { body } }
// alias path "pd.cmt" must be translated to schema path "postDetail.comments"
// for the row policy lookup to apply the org filter correctly.
#[tokio::test]
async fn deep_nested_alias_translates_to_schema_path() -> Res<()> {
    let pol = row_policy("postDetail.comments".to_owned(), "any".to_owned());
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_relation_setup(pol, cfg).await?;

    let q = "
    query($id: ID!) {
        pd: postDetail(id: $id) {
            cmt: comments(orderBy: [BodyAsc]) {
                body
            }
        }
    }
    ";
    let expected = value!({
        "pd": {
            "cmt": [{
                "body": "Anomaly detected",
            }],
        },
    });
    exec_assert_id(&d.schema, q, &d.post1_id, &expected).await;

    d.tmp.drop().await
}

// Two root ops each with a nested relation in one request.
// postDetail.comments walk-up finds "postDetail", commentDetail.post walk-up
// finds "commentDetail". Neither crosses into the other root's cache entry.
// OrgHandler filters by org1, so only org1 records pass.
#[tokio::test]
async fn nested_two_ops_each_isolated() -> Res<()> {
    let pol = hashmap! {
        "postDetail.comments".to_owned() => "any".to_owned(),
        "commentDetail.post".to_owned() => "any".to_owned(),
    };
    let cfg = AuthzConfig {
        handlers: Arc::new(OrgHandler),
        ..Default::default()
    };
    let d = row_relation_setup(pol, cfg).await?;

    let q = "
    query($pid: ID!, $cid: ID!) {
        postDetail(id: $pid) {
            comments(orderBy: [BodyAsc]) {
                body
            }
        }
        commentDetail(id: $cid) {
            post {
                id
            }
        }
    }
    ";
    let v = value!({
        "pid": d.post1_id,
        "cid": d.comment_a_id,
    });
    let expected = value!({
        "postDetail": {
            "comments": [{
                "body": "Anomaly detected",
            }],
        },
        "commentDetail": {
            "post": {
                "id": d.post1_id,
            },
        },
    });
    exec_assert(&d.schema, q, Some(v), &expected).await;

    d.tmp.drop().await
}
