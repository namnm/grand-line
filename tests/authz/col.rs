#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn allowed_field_returns_value() -> Res<()> {
    let d = setup_with_col_policy(col_policy_with_children("org", "name")).await?;

    let h = auth_headers(d.h, &d.org_id1, &d.token1, &d.role_id1);

    let s = d.s.data(h).finish();

    // name is in the allowed field map -> ok.
    let q = "
    query test {
        org {
            name
        }
    }
    ";
    let expected = value!({
        "org": {
            "name": "Fringe",
        },
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn denied_field_returns_unauthorized() -> Res<()> {
    let d = setup_with_col_policy(col_policy_with_children("org", "name")).await?;

    let h = auth_headers(d.h, &d.org_id1, &d.token1, &d.role_id1);

    let s = d.s.data(h).finish();

    // id is not in the allowed field map -> unauthorized.
    let q = "
    query test {
        org {
            id
        }
    }
    ";
    exec_assert_err(&s, q, None, &AuthzErr::Unauthorized).await?;

    // Selecting both an allowed (name) and a denied (id) field -> unauthorized.
    let q = "
    query test {
        org {
            id
            name
        }
    }
    ";
    exec_assert_err(&s, q, None, &AuthzErr::Unauthorized).await?;

    // orgPrimitive is not in the allowed operation map -> unauthorized.
    let q = "
    query test {
        orgPrimitive
    }
    ";
    exec_assert_err(&s, q, None, &AuthzErr::Unauthorized).await?;

    d.tmp.drop().await
}

// Col policy checks the schema field name, not the alias used in the query.
// myAlias: name -> schema name is "name" -> col policy allows "name" -> ok.
// This verifies that aliasing a field cannot be used to bypass column policy.
#[tokio::test]
async fn alias_on_output_field_uses_schema_name() -> Res<()> {
    let d = setup_with_col_policy(col_policy_with_children("org", "name")).await?;

    let h = auth_headers(d.h, &d.org_id1, &d.token1, &d.role_id1);

    let s = d.s.data(h).finish();

    // alias "myName" maps to schema field "name", policy checks "name" -> allowed.
    let q = "
    query {
        org {
            myName: name
        }
    }
    ";
    let expected = value!({
        "org": {
            "myName": "Fringe",
        },
    });
    exec_assert(&s, q, None, &expected).await;

    // alias "myId" maps to schema field "id", policy does not allow "id" -> unauthorized.
    let q = "
    query {
        org {
            myId: id
        }
    }
    ";
    exec_assert_err(&s, q, None, &AuthzErr::Unauthorized).await?;

    d.tmp.drop().await
}
