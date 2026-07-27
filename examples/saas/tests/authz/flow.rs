#[path = "./setup.rs"]
mod setup;
use setup::*;

// ---------------------------------------------------------------------------
// Full org admin flow: create a role, assign a member, invite another member,
// impersonate one of them, then revoke the impersonation.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn org_admin_creates_role_invites_and_impersonates_a_member() -> Res<()> {
    let d = setup().await?;
    let acme = Org::find()
        .filter(OrgColumn::Name.eq("Acme Inc"))
        .one_or_404(&d.tmp.db)
        .await?;
    let bootstrap_role = seeded_bootstrap_role(&d.tmp).await?;
    let admin_bearer = login_bearer(&d, "system@example.com", "123123").await?;

    let mut h_admin = d.h.clone();
    h_admin.insert(H_AUTHORIZATION, admin_bearer);
    let h_admin = h_authz(h_admin, &acme.id, &bootstrap_role.id);

    // -----------------------------------------------------------------------
    // roleCreate: the org admin provisions a new "Fringe Analyst" role
    // -----------------------------------------------------------------------

    let s = d.schema(h_admin.clone());
    let q = r#"
    mutation($colPolicy: JSON!, $rowPolicy: JSON!) {
        roleCreate(data: { name: "Fringe Analyst", colPolicy: $colPolicy, rowPolicy: $rowPolicy }) {
            id
            name
            orgId
        }
    }
    "#;
    let v = value!({
        "colPolicy": wildcard_col_policy(),
        "rowPolicy": json!({}),
    });
    let res = exec_assert_ok(&s, q, Some(v)).await;
    let data = json_data(&res);
    let role_id = data.str("/roleCreate/id").to_owned();
    pretty_eq!(
        data.str("/roleCreate/orgId"),
        acme.id,
        "the new role should be scoped to Acme Inc"
    );

    // -----------------------------------------------------------------------
    // userInRoleCreate: assign Peter Bishop into that role
    // -----------------------------------------------------------------------

    let (peter_id, peter_bearer) =
        register_and_resolve(&d, "peter@fringe.example", "Amber-Universe-Bishop-11!", "205551").await?;

    let s = d.schema(h_admin.clone());
    let q = "
    mutation($userId: String!, $roleId: String!) {
        userInRoleCreate(data: { userId: $userId, roleId: $roleId }) {
            userId
            roleId
            orgId
        }
    }
    ";
    let v = value!({
        "userId": peter_id,
        "roleId": role_id,
    });
    let expected = value!({
        "userInRoleCreate": {
            "userId": peter_id,
            "roleId": role_id,
            "orgId": acme.id,
        },
    });
    exec_assert(&s, q, Some(v), &expected).await;

    // -----------------------------------------------------------------------
    // roleUpdate: rename the role, roleDetail should reflect the rename
    // -----------------------------------------------------------------------

    let s = d.schema(h_admin.clone());
    let q = r#"
    mutation($id: String!, $colPolicy: JSON!, $rowPolicy: JSON!) {
        roleUpdate(id: $id, data: { name: "Senior Fringe Analyst", colPolicy: $colPolicy, rowPolicy: $rowPolicy }) {
            id
        }
    }
    "#;
    let v = value!({
        "id": role_id,
        "colPolicy": wildcard_col_policy(),
        "rowPolicy": json!({}),
    });
    exec_assert_ok(&s, q, Some(v)).await;

    let s = d.schema(h_admin.clone());
    let q = "
    query($id: ID!) {
        roleDetail(id: $id) {
            name
        }
    }
    ";
    let expected = value!({
        "roleDetail": {
            "name": "Senior Fringe Analyst",
        },
    });
    exec_assert_id(&s, q, &role_id, &expected).await;

    // -----------------------------------------------------------------------
    // invitationCreate + orgInvitationResolve: invite Astrid into the same role
    // -----------------------------------------------------------------------

    let (astrid_id, astrid_bearer) =
        register_and_resolve(&d, "astrid@fringe.example", "Farnsworth-Lab-Assistant-77!", "884422").await?;

    let s = d.schema(h_admin.clone());
    let q = "
    mutation($email: Email!, $roleId: String!) {
        invitationCreate(data: { email: $email, roleId: $roleId }) {
            id
            secret
        }
    }
    ";
    let v = value!({
        "email": "astrid@fringe.example",
        "roleId": role_id,
    });
    let res = exec_assert_ok(&s, q, Some(v)).await;
    let data = json_data(&res);
    let invitation_id = data.str("/invitationCreate/id").to_owned();
    let invitation_secret = data.str("/invitationCreate/secret").to_owned();

    let mut h_astrid = d.h.clone();
    h_astrid.insert(H_AUTHORIZATION, astrid_bearer);

    let s = d.schema(h_astrid.clone());
    let q = "
    query {
        myOrgInvitations {
            orgId
            roleId
        }
    }
    ";
    let expected = value!({
        "myOrgInvitations": [
            {
                "orgId": acme.id,
                "roleId": role_id,
            },
        ],
    });
    exec_assert(&s, q, None, &expected).await;

    known_otp_for(&d.tmp, &invitation_id, "112233").await?;

    let s = d.schema(h_astrid.clone());
    let q = "
    mutation($id: String!, $secret: String!, $otp: String!) {
        orgInvitationResolve(data: { id: $id, secret: $secret, otp: $otp }) {
            userId
            roleId
            orgId
        }
    }
    ";
    let v = value!({
        "id": invitation_id,
        "secret": invitation_secret,
        "otp": "112233",
    });
    let expected = value!({
        "orgInvitationResolve": {
            "userId": astrid_id,
            "roleId": role_id,
            "orgId": acme.id,
        },
    });
    exec_assert(&s, q, Some(v), &expected).await;

    // -----------------------------------------------------------------------
    // impersonate Peter, use the impersonated session, then revoke it
    // -----------------------------------------------------------------------

    let s = d.schema(h_admin.clone());
    let q = "
    mutation($userId: String!, $reason: String!) {
        impersonate(userId: $userId, reason: $reason) {
            id
            secret
        }
    }
    ";
    let v = value!({
        "userId": peter_id,
        "reason": "Routine Fringe Division support session",
    });
    let res = exec_assert_ok(&s, q, Some(v)).await;
    let data = json_data(&res);
    let impersonated_session_id = data.str("/impersonate/id").to_owned();
    let impersonated_session_secret = data.str("/impersonate/secret").to_owned();
    let impersonated_bearer = h_bearer_for(&impersonated_session_id, &impersonated_session_secret);

    let mut h = d.h.clone();
    h.insert(H_AUTHORIZATION, impersonated_bearer.clone());
    let s = d.schema(h);
    let q = "
    query {
        loginSessionCurrent {
            userId
        }
    }
    ";
    let expected = value!({
        "loginSessionCurrent": {
            "userId": peter_id,
        },
    });
    exec_assert(&s, q, None, &expected).await;

    let imp = Impersonation::find()
        .filter(ImpersonationColumn::LoginSessionId.eq(&impersonated_session_id))
        .one_or_404(&d.tmp.db)
        .await?;

    let s = d.schema(h_admin.clone());
    let q = "
    mutation($id: String!) {
        impersonateRevoke(id: $id) {
            id
        }
    }
    ";
    let v = value!({
        "id": imp.id,
    });
    exec_assert_ok(&s, q, Some(v)).await;

    let mut h = d.h.clone();
    h.insert(H_AUTHORIZATION, impersonated_bearer);
    let s = d.schema(h);
    let q = "
    query {
        loginSessionCurrent {
            userId
        }
    }
    ";
    let expected = value!({
        "loginSessionCurrent": null,
    });
    exec_assert(&s, q, None, &expected).await;

    // Peter's own (non-impersonated) session must survive the revoke.
    let mut h = d.h.clone();
    h.insert(H_AUTHORIZATION, peter_bearer);
    let s = d.schema(h);
    let q = "
    query {
        loginSessionCurrent {
            userId
        }
    }
    ";
    let expected = value!({
        "loginSessionCurrent": {
            "userId": peter_id,
        },
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}
