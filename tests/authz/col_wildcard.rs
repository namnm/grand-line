#[path = "./setup.rs"]
mod setup;
use setup::*;

// ---------------------------------------------------------------------------
// Wildcard access for both realms
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ok_org_realm() -> Res<()> {
    let d = setup_with_col_wildcard().await?;

    let h = auth_headers(d.h, &d.org_id1, &d.token1, &d.role_id1);

    let s = d.s.data(h).finish();

    let q = "
    query test {
        orgPrimitive
    }
    ";
    exec_assert_ok(&s, q, None).await;

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
async fn ok_system_realm() -> Res<()> {
    let d = setup_with_col_wildcard().await?;

    let mut h = d.h;
    h.insert(H_AUTHORIZATION, h_bearer(&d.token1));
    h.insert(H_ROLE_ID, h_str(&d.role_id1_system));

    let s = d.s.data(h).finish();

    let q = "
    query test {
        systemPrimitive
    }
    ";
    exec_assert_ok(&s, q, None).await;

    let q = "
    query test($orgId: String!) {
        system(orgId: $orgId) {
            name
        }
    }
    ";
    let v = value!({
        "orgId": d.org_id2,
    });
    let expected = value!({
        "system": {
            "name": "FBI",
        },
    });
    exec_assert(&s, q, Some(v), &expected).await;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// Cross-org role rejection
// ---------------------------------------------------------------------------

#[tokio::test]
async fn role_from_other_org_returns_unauthorized() -> Res<()> {
    let d = setup_with_col_wildcard().await?;

    // user2's role (role_id2) belongs to org2. Sending org1 header -> role not found
    // (role's OrgId != org1) -> Unauthorized.
    let h = auth_headers(d.h, &d.org_id1, &d.token2, &d.role_id2);

    let s = d.s.data(h).finish();

    let q = "
    query test {
        org {
            name
        }
    }
    ";
    exec_assert_err(&s, q, None, &AuthzErr::Unauthorized).await?;

    d.tmp.drop().await
}
