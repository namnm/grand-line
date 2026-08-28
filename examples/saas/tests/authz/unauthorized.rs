#[path = "./setup.rs"]
mod setup;
use setup::*;

// ---------------------------------------------------------------------------
// check = authz_org header/role gaps
// ---------------------------------------------------------------------------

#[tokio::test]
async fn role_create_without_org_header_fails() -> Res<()> {
    let d = setup().await?;
    let bootstrap_role = seeded_bootstrap_role(&d.tmp).await?;
    let admin_bearer = login_bearer(&d, "system@example.com", "123123").await?;

    let mut h = d.h.clone();
    h.insert(H_AUTHORIZATION, admin_bearer);
    h.insert(H_ROLE_ID, h_str(&bootstrap_role.id));
    let s = d.schema(h);
    let q = r#"
    mutation($colPolicy: JSON!, $rowPolicy: JSON!) {
        roleCreate(data: { name: "Fringe Analyst", colPolicy: $colPolicy, rowPolicy: $rowPolicy }) {
            id
        }
    }
    "#;
    let v = value!({
        "colPolicy": wildcard_col_policy(),
        "rowPolicy": json!({}),
    });
    exec_assert_err(&s, q, Some(v), &AuthzErr::HeaderOrgId404).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn role_create_without_role_header_fails() -> Res<()> {
    let d = setup().await?;
    let acme = Org::find()
        .filter(OrgColumn::Name.eq("Acme Inc"))
        .one_or_404(&d.tmp.db)
        .await?;
    let admin_bearer = login_bearer(&d, "system@example.com", "123123").await?;

    let mut h = d.h.clone();
    h.insert(H_AUTHORIZATION, admin_bearer);
    h.insert(H_ORG_ID, h_str(&acme.id));
    let s = d.schema(h);
    let q = r#"
    mutation($colPolicy: JSON!, $rowPolicy: JSON!) {
        roleCreate(data: { name: "Fringe Analyst", colPolicy: $colPolicy, rowPolicy: $rowPolicy }) {
            id
        }
    }
    "#;
    let v = value!({
        "colPolicy": wildcard_col_policy(),
        "rowPolicy": json!({}),
    });
    exec_assert_err(&s, q, Some(v), &AuthzErr::HeaderRoleId404).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn role_create_with_a_role_the_caller_is_not_assigned_to_fails() -> Res<()> {
    let d = setup().await?;
    let acme = Org::find()
        .filter(OrgColumn::Name.eq("Acme Inc"))
        .one_or_404(&d.tmp.db)
        .await?;
    let bootstrap_role = seeded_bootstrap_role(&d.tmp).await?;

    // Peter is a real, registered user, but never assigned to any role.
    let (_, peter_bearer) =
        register_and_resolve(&d, "peter@fringe.example", "Amber-Universe-Bishop-11!", "205551").await?;

    let mut h = d.h.clone();
    h.insert(H_AUTHORIZATION, peter_bearer);
    let h = h_authz(h, &acme.id, &bootstrap_role.id);
    let s = d.schema(h);
    let q = r#"
    mutation($colPolicy: JSON!, $rowPolicy: JSON!) {
        roleCreate(data: { name: "Fringe Analyst", colPolicy: $colPolicy, rowPolicy: $rowPolicy }) {
            id
        }
    }
    "#;
    let v = value!({
        "colPolicy": wildcard_col_policy(),
        "rowPolicy": json!({}),
    });
    exec_assert_err(&s, q, Some(v), &AuthzErr::Unauthorized).await?;

    d.tmp.drop().await
}
