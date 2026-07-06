// Tests that authz-protected resolvers reject requests with no auth token.
// role_id must be provided so the authz check reaches the user verification step.

#[path = "./setup.rs"]
mod setup;
use setup::*;

// A request with no Authorization header to an org-realm resolver returns Unauthenticated.
#[tokio::test]
async fn no_token_org_realm() -> Res<()> {
    let d = setup_with_col_wildcard().await?;

    let mut h = d.h;
    h.append(H_ORG_ID, h_str(&d.org_id1));
    h.insert(H_ROLE_ID, h_str(&d.role_id1));
    // Intentionally omit H_AUTHORIZATION.
    let s = d.s.data(h).finish();

    let q = "
    query test {
        orgPrimitive
    }
    ";
    exec_assert_err(&s, q, None, &AuthErr::Unauthenticated).await?;

    d.tmp.drop().await
}

// A request with no Authorization header to a system-realm resolver returns Unauthenticated.
#[tokio::test]
async fn no_token_system_realm() -> Res<()> {
    let d = setup_with_col_wildcard().await?;

    let mut h = d.h;
    h.insert(H_ROLE_ID, h_str(&d.role_id1_system));
    // Intentionally omit H_AUTHORIZATION.
    let s = d.s.data(h).finish();

    let q = "
    query test {
        systemPrimitive
    }
    ";
    exec_assert_err(&s, q, None, &AuthErr::Unauthenticated).await?;

    d.tmp.drop().await
}
