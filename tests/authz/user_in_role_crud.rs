#[path = "./resolvers_setup.rs"]
mod resolvers_setup;
use resolvers_setup::*;

#[tokio::test]
async fn org_user_in_role_create_and_search_scoped_to_own_org() -> Res<()> {
    let d = resolvers_setup().await?;
    let h = auth_headers(d.h, &d.org_id1, &d.token1, &d.role_id1);
    let s = d.s.data(h).finish();

    let v = value!({
        "data": {
            "userId": d.user_id2,
            "roleId": d.role_id1,
        },
    });
    exec_assert_ok(
        &s,
        "mutation t($data: OrgUserInRoleCreate!) { orgUserInRoleCreate(data: $data) { userId roleId } }",
        Some(v),
    )
    .await;

    let q = "query { orgUserInRoleCount }";
    let expected = value!({
        "orgUserInRoleCount": 2,
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn org_user_in_role_delete_denies_cross_org() -> Res<()> {
    let d = resolvers_setup().await?;
    let h = auth_headers(d.h, &d.org_id1, &d.token1, &d.role_id1);
    let s = d.s.data(h).finish();

    // role_id2 belongs to org2, so no UserInRole row for it exists in org1.
    let q = "mutation t($id: ID!) { orgUserInRoleDelete(id: $id) { id } }";
    exec_assert_err_id(&s, q, &d.role_id2, &CoreDbErr::Db404).await?;

    d.tmp.drop().await
}

// system realm is intentionally read-only for UserInRole: no create/delete mutation
// should be reachable at all, only search/count/detail.
#[tokio::test]
async fn system_realm_has_no_user_in_role_mutations() -> Res<()> {
    let d = resolvers_setup().await?;
    let s = d.s.finish();

    let sdl = s.sdl();
    pretty_eq!(
        sdl.contains("systemUserInRoleCreate"),
        false,
        "systemUserInRoleCreate mutation should not exist",
    );
    pretty_eq!(
        sdl.contains("systemUserInRoleDelete"),
        false,
        "systemUserInRoleDelete mutation should not exist",
    );

    d.tmp.drop().await
}
