#[path = "./resolvers_setup.rs"]
mod resolvers_setup;
use resolvers_setup::*;

pub struct MockAuthHandlers;
#[async_trait]
impl AuthHandlers for MockAuthHandlers {
    async fn otp(&self, _ctx: &Context<'_>) -> Res<String> {
        Ok("999999".to_owned())
    }
}

#[tokio::test]
async fn invitation_create_then_accept_creates_user_in_role() -> Res<()> {
    let d = resolvers_setup().await?;

    // The invited person is an existing app user, not yet a member of org1.
    let u3 = am_create!(User {
        email: "walter@example.com",
        password_hashed: rand_utils::password_hash("123123")?,
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;
    let ua = Context::get_ua_raw(Context::axum_headers(&init_common_headers()))?;
    let secret3 = rand_utils::secret();
    let ls3 = am_create!(LoginSession {
        user_id: u3.id.clone(),
        secret_hashed: rand_utils::secret_hash(&secret3),
        ip: "127.0.0.1",
        ua: ua.to_json()?,
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;
    let token3 = rand_utils::qs_token(&ls3.id, &secret3)?;

    // user1 (org1 admin) invites walter@example.com into org1 with role_id1.
    let auth_cfg = AuthConfig {
        handlers: Arc::new(MockAuthHandlers),
        ..Default::default()
    };
    let h1 = auth_headers(d.h.clone(), &d.org_id1, &d.token1, &d.role_id1);
    let s1 = d.s.data(auth_cfg).data(h1).finish();

    let v = value!({
        "data": {
            "email": "walter@example.com",
            "roleId": d.role_id1,
        },
    });
    let r = exec_assert_ok(
        &s1,
        "mutation t($data: OrgInvitationCreate!) { orgInvitationCreate(data: $data) { secret inner { id } } }",
        Some(v),
    )
    .await;
    let r = r.data.to_json()?;
    let id = r.str("/orgInvitationCreate/inner/id");
    let secret = r.str("/orgInvitationCreate/secret");

    // walter accepts, logged in as himself, using the id/secret plus the known otp code.
    let h3 = auth_headers(init_common_headers(), &d.org_id1, &token3, &d.role_id1);
    let s3 = schema_qm::<Query, Mutation>(&d.tmp.db)
        .data(Org::authz_default_impl())
        .data(h3)
        .finish();

    let v = value!({
        "data": {
            "id": id,
            "secret": secret,
            "otp": "999999",
        },
    });
    let expected = value!({
        "orgInvitationResolve": {
            "userId": u3.id,
            "roleId": d.role_id1,
            "orgId": d.org_id1,
        },
    });
    exec_assert(
        &s3,
        "mutation t($data: OtpResolve!) { orgInvitationResolve(data: $data) { userId roleId orgId } }",
        Some(v),
        &expected,
    )
    .await;

    let uir = UserInRole::find()
        .filter(UserInRoleColumn::UserId.eq(&u3.id))
        .filter(UserInRoleColumn::OrgId.eq(&d.org_id1))
        .one(&d.tmp.db)
        .await?;
    pretty_eq!(
        uir.is_some(),
        true,
        "UserInRole should be created after accepting the invitation"
    );

    let otp_gone = Otp::find().one(&d.tmp.db).await?;
    pretty_eq!(
        otp_gone.is_none(),
        true,
        "Otp row should be deleted after accepting the invitation"
    );

    d.tmp.drop().await
}

#[tokio::test]
async fn invitation_reject_deletes_otp_without_creating_user_in_role() -> Res<()> {
    let d = resolvers_setup().await?;

    let auth_cfg = AuthConfig {
        handlers: Arc::new(MockAuthHandlers),
        ..Default::default()
    };
    let h1 = auth_headers(d.h.clone(), &d.org_id1, &d.token1, &d.role_id1);
    let s1 = d.s.data(auth_cfg).data(h1).finish();

    let v = value!({
        "data": {
            "email": "walter@example.com",
            "roleId": d.role_id1,
        },
    });
    let r = exec_assert_ok(
        &s1,
        "mutation t($data: OrgInvitationCreate!) { orgInvitationCreate(data: $data) { secret inner { id } } }",
        Some(v),
    )
    .await;
    let r = r.data.to_json()?;
    let id = r.str("/orgInvitationCreate/inner/id");
    let secret = r.str("/orgInvitationCreate/secret");

    let v = value!({
        "data": {
            "id": id,
            "secret": secret,
            "otp": "999999",
        },
    });
    exec_assert_ok(
        &s1,
        "mutation t($data: OtpResolve!) { orgInvitationReject(data: $data) { id } }",
        Some(v),
    )
    .await;

    let otp_gone = Otp::find().one(&d.tmp.db).await?;
    pretty_eq!(
        otp_gone.is_none(),
        true,
        "Otp row should be deleted after rejecting the invitation"
    );

    // 3 seeded rows: user1-role1(org1), user2-role2(org2), user1-role3(system).
    let uir_count = UserInRole::find().count(&d.tmp.db).await?;
    pretty_eq!(
        uir_count,
        3,
        "no new UserInRole should be created by a reject beyond the seeded rows"
    );

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// org_invitation_resolve_by_email: for host apps chaining invitation resolution
// into their own AuthHandlers::on_register_resolve, when the invited email has
// no account yet at invite time.
// ---------------------------------------------------------------------------

#[derive(Default, MergedObject)]
pub struct FullQuery(AuthzMergedQuery<User>);
#[derive(Default, MergedObject)]
pub struct FullMutation(AuthMergedMutation<User>, AuthzMergedMutation<User>);

pub struct ChainedAuthHandlers;
#[async_trait]
impl AuthHandlers for ChainedAuthHandlers {
    async fn otp(&self, _ctx: &Context<'_>) -> Res<String> {
        Ok("999999".to_owned())
    }
    async fn on_register_resolve(&self, ctx: &Context<'_>, user_id: &str, _ls: &LoginSessionSql) -> Res<()> {
        org_invitation_resolve_by_email::<User>(ctx, user_id).await?;
        Ok(())
    }
}

#[tokio::test]
async fn org_invitation_resolve_by_email_auto_joins_new_registrant() -> Res<()> {
    let d = resolvers_setup().await?;

    // user1 (org1 admin) invites a brand-new email, unaware it has no account yet.
    let auth_cfg = AuthConfig {
        handlers: Arc::new(MockAuthHandlers),
        ..Default::default()
    };
    let h1 = auth_headers(d.h.clone(), &d.org_id1, &d.token1, &d.role_id1);
    let s1 = d.s.data(auth_cfg).data(h1).finish();

    let v = value!({
        "data": {
            "email": "astrid@example.com",
            "roleId": d.role_id1,
        },
    });
    exec_assert_ok(
        &s1,
        "mutation t($data: OrgInvitationCreate!) { orgInvitationCreate(data: $data) { secret } }",
        Some(v),
    )
    .await;

    // astrid registers with no knowledge of the invitation at all, the host app's
    // on_register_resolve hook chains org_invitation_resolve_by_email automatically.
    let s2 = schema_qm::<FullQuery, FullMutation>(&d.tmp.db)
        .data(Org::authz_default_impl())
        .data(AuthConfig {
            handlers: Arc::new(ChainedAuthHandlers),
            ..Default::default()
        })
        .data(init_common_headers())
        .finish();

    let v = value!({
        "data": {
            "email": "astrid@example.com",
            "password": "Str0ngP@ssw0rd?",
        },
    });
    let r = exec_assert_ok(
        &s2,
        "mutation t($data: Register!) { register(data: $data) { secret inner { id } } }",
        Some(v),
    )
    .await;
    let r = r.data.to_json()?;
    let otp_id = r.str("/register/inner/id");
    let otp_secret = r.str("/register/secret");

    let v = value!({
        "data": {
            "id": otp_id,
            "secret": otp_secret,
            "otp": "999999",
        },
    });
    exec_assert_ok(
        &s2,
        "mutation t($data: OtpResolve!) { registerResolve(data: $data) { inner { userId } } }",
        Some(v),
    )
    .await;

    let new_user = User::find()
        .filter(UserColumn::Email.eq("astrid@example.com"))
        .one(&d.tmp.db)
        .await?;
    let Some(new_user) = new_user else {
        return TestErr::expect("astrid's User row should exist after registering");
    };

    let uir = UserInRole::find()
        .filter(UserInRoleColumn::UserId.eq(&new_user.id))
        .filter(UserInRoleColumn::OrgId.eq(&d.org_id1))
        .one(&d.tmp.db)
        .await?;
    pretty_eq!(
        uir.is_some(),
        true,
        "UserInRole should be auto-created via the on_register_resolve hook chain",
    );

    let otp_gone = Otp::find().one(&d.tmp.db).await?;
    pretty_eq!(
        otp_gone.is_none(),
        true,
        "the invitation's Otp row should be consumed by the auto-resolve",
    );

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// myOrgInvitations: lets an already-authenticated user see and act on their
// own pending invitations.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn my_org_invitations_lists_pending_invites_for_current_user() -> Res<()> {
    let d = resolvers_setup().await?;

    // user1 (org1 admin) invites user2 (peter@example.com, already has an account) into org1.
    let auth_cfg = AuthConfig {
        handlers: Arc::new(MockAuthHandlers),
        ..Default::default()
    };
    let h1 = auth_headers(d.h.clone(), &d.org_id1, &d.token1, &d.role_id1);
    let s1 = d.s.data(auth_cfg).data(h1).finish();

    let v = value!({
        "data": {
            "email": "peter@example.com",
            "roleId": d.role_id1,
        },
    });
    exec_assert_ok(
        &s1,
        "mutation t($data: OrgInvitationCreate!) { orgInvitationCreate(data: $data) { secret } }",
        Some(v),
    )
    .await;

    // user2 (peter), already authenticated in their own org2, checks their invitations.
    let h2 = auth_headers(init_common_headers(), &d.org_id2, &d.token2, &d.role_id2);
    let s2 = schema_qm::<Query, Mutation>(&d.tmp.db)
        .data(Org::authz_default_impl())
        .data(h2)
        .finish();

    let q = "query { myOrgInvitations { orgId roleId } }";
    let expected = value!({
        "myOrgInvitations": [
            { "orgId": d.org_id1, "roleId": d.role_id1 },
        ],
    });
    exec_assert(&s2, q, None, &expected).await;

    d.tmp.drop().await
}
