#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn err_on_missing_role_id() -> Res<()> {
    let d = setup_with_col_wildcard().await?;

    let mut h = d.h;
    h.insert(H_AUTHORIZATION, h_bearer(&d.token1));
    // Intentionally omit H_ROLE_ID.
    let s = d.s.data(h).finish();

    let q = "
    query test {
        systemPrimitive
    }
    ";
    exec_assert_err(&s, q, None, &AuthzErr::HeaderRoleId404).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn err_on_missing_org_id() -> Res<()> {
    let d = setup_with_col_wildcard().await?;

    let mut h = d.h;
    h.insert(H_AUTHORIZATION, h_bearer(&d.token1));
    h.insert(H_ROLE_ID, h_str(&d.role_id1));
    // Intentionally omit H_ORG_ID.
    let s = d.s.data(h).finish();

    let q = "
    query test {
        orgPrimitive
    }
    ";
    exec_assert_err(&s, q, None, &AuthzErr::HeaderOrgId404).await?;

    d.tmp.drop().await
}
