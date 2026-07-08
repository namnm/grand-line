#[path = "./resolvers_setup.rs"]
mod resolvers_setup;
use resolvers_setup::*;

// ---------------------------------------------------------------------------
// org realm: scoped to the caller's own org
// ---------------------------------------------------------------------------

#[tokio::test]
async fn org_role_create_and_search_scoped_to_own_org() -> Res<()> {
    let d = resolvers_setup().await?;
    let h = auth_headers(d.h, &d.org_id1, &d.token1, &d.role_id1);
    let s = d.s.data(h).finish();

    let v = value!({
        "data": {
            "name": "Support",
            "colPolicy": {},
            "rowPolicy": {},
        },
    });
    exec_assert_ok(
        &s,
        "mutation t($data: OrgRoleCreate!) { orgRoleCreate(data: $data) { id name } }",
        Some(v),
    )
    .await;

    let q = "query { orgRoleSearch(orderBy: [NameAsc]) { name } }";
    let expected = value!({
        "orgRoleSearch": [
            { "name": "Org Admin" },
            { "name": "Support" },
        ],
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn org_role_update_denies_cross_org_role() -> Res<()> {
    let d = resolvers_setup().await?;
    let h = auth_headers(d.h, &d.org_id1, &d.token1, &d.role_id1);
    let s = d.s.data(h).finish();

    let q = r#"
    mutation t($id: ID!) {
        orgRoleUpdate(id: $id, data: { name: "Hacked", colPolicy: {}, rowPolicy: {} }) {
            id
        }
    }
    "#;
    exec_assert_err_id(&s, q, &d.role_id2, &CoreDbErr::Db404).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn org_role_delete_denies_cross_org_role() -> Res<()> {
    let d = resolvers_setup().await?;
    let h = auth_headers(d.h, &d.org_id1, &d.token1, &d.role_id1);
    let s = d.s.data(h).finish();

    let q = "mutation t($id: ID!) { orgRoleDelete(id: $id) { id } }";
    exec_assert_err_id(&s, q, &d.role_id2, &CoreDbErr::Db404).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn org_role_delete_own_org_role_succeeds() -> Res<()> {
    let d = resolvers_setup().await?;
    let h = auth_headers(d.h, &d.org_id1, &d.token1, &d.role_id1);
    let s = d.s.data(h).finish();

    let q = "mutation t($id: ID!) { orgRoleDelete(id: $id) { id } }";
    let expected = value!({
        "orgRoleDelete": {
            "id": d.role_id1,
        },
    });
    exec_assert_id(&s, q, &d.role_id1, &expected).await;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// system realm: unrestricted across every org
// ---------------------------------------------------------------------------

#[tokio::test]
async fn system_role_search_sees_every_org() -> Res<()> {
    let d = resolvers_setup().await?;
    let h = auth_headers(d.h, &d.org_id1, &d.token1, &d.role_id1_system);
    let s = d.s.data(h).finish();

    // 3 roles seeded total: org1's Org Admin, org2's Org Admin, System Admin.
    let q = "query { systemRoleCount }";
    let expected = value!({
        "systemRoleCount": 3,
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn system_role_create_update_delete_any_org() -> Res<()> {
    let d = resolvers_setup().await?;
    let h = auth_headers(d.h, &d.org_id1, &d.token1, &d.role_id1_system);
    let s = d.s.data(h).finish();

    let v = value!({
        "data": {
            "name": "Auditor",
            "realm": "org",
            "orgId": d.org_id2,
            "colPolicy": {},
            "rowPolicy": {},
        },
    });
    let r = exec_assert_ok(
        &s,
        "mutation t($data: SystemRoleCreate!) { systemRoleCreate(data: $data) { id } }",
        Some(v),
    )
    .await;
    let r = r.data.to_json()?;
    let id = r.str("/systemRoleCreate/id");

    let q = r#"
    mutation t($id: ID!) {
        systemRoleUpdate(id: $id, data: { name: "Auditor2", colPolicy: {}, rowPolicy: {} }) {
            name
        }
    }
    "#;
    let expected = value!({
        "systemRoleUpdate": {
            "name": "Auditor2",
        },
    });
    exec_assert_id(&s, q, id, &expected).await;

    let q = "mutation t($id: ID!) { systemRoleDelete(id: $id) { id } }";
    let expected = value!({
        "systemRoleDelete": {
            "id": id,
        },
    });
    exec_assert_id(&s, q, id, &expected).await;

    d.tmp.drop().await
}
