#[path = "./resolvers_setup.rs"]
mod resolvers_setup;
use resolvers_setup::*;

const Q_IMPERSONATE: &str = "
mutation t($userId: String!, $reason: String!) {
    orgImpersonate(userId: $userId, reason: $reason) {
        secret
        inner { id userId }
    }
}
";

#[tokio::test]
async fn org_impersonate_own_org_user_records_admin_as_creator() -> Res<()> {
    let d = resolvers_setup().await?;
    let h = auth_headers(d.h.clone(), &d.org_id1, &d.token1, &d.role_id1);
    let s = d.s.data(h).finish();

    let v = value!({
        "userId": d.user_id1,
        "reason": "support ticket #42",
    });
    let r = exec_assert_ok(&s, Q_IMPERSONATE, Some(v)).await;
    let r = r.data.to_json()?;
    let ls_id = r.str("/orgImpersonate/inner/id");
    pretty_eq!(
        ls_id.is_empty(),
        false,
        "impersonation login session id should be returned"
    );

    let imp = Impersonation::find()
        .filter(ImpersonationColumn::LoginSessionId.eq(ls_id))
        .one(&d.tmp.db)
        .await?;
    let Some(imp) = imp else {
        return TestErr::expect("Impersonation row should be created");
    };
    pretty_eq!(
        imp.created_by_id,
        Some(d.user_id1.clone()),
        "Impersonation created_by_id should be the impersonating admin",
    );
    pretty_eq!(
        imp.user_id,
        d.user_id1,
        "Impersonation user_id should be the impersonated user"
    );
    pretty_eq!(
        imp.org_id,
        Some(d.org_id1.clone()),
        "org-scoped impersonation should record org_id"
    );

    d.tmp.drop().await
}

#[tokio::test]
async fn org_impersonate_denies_user_of_another_org() -> Res<()> {
    let d = resolvers_setup().await?;
    let h = auth_headers(d.h.clone(), &d.org_id1, &d.token1, &d.role_id1);
    let s = d.s.data(h).finish();

    let v = value!({
        "userId": d.user_id2,
        "reason": "should not be allowed",
    });
    exec_assert_err(&s, Q_IMPERSONATE, Some(v), &CoreDbErr::Db404).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn system_impersonate_any_org_user() -> Res<()> {
    let d = resolvers_setup().await?;
    let h = auth_headers(d.h.clone(), &d.org_id1, &d.token1, &d.role_id1_system);
    let s = d.s.data(h).finish();

    let q = "
    mutation t($userId: String!, $reason: String!) {
        systemImpersonate(userId: $userId, reason: $reason) {
            inner { userId }
        }
    }
    ";
    let v = value!({
        "userId": d.user_id2,
        "reason": "support ticket #43",
    });
    let expected = value!({
        "systemImpersonate": {
            "inner": {
                "userId": d.user_id2,
            },
        },
    });
    exec_assert(&s, q, Some(v), &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn org_impersonate_revoke_deletes_session_and_soft_deletes_record() -> Res<()> {
    let d = resolvers_setup().await?;
    let h = auth_headers(d.h.clone(), &d.org_id1, &d.token1, &d.role_id1);
    let s = d.s.data(h).finish();

    let v = value!({
        "userId": d.user_id1,
        "reason": "support ticket #44",
    });
    let r = exec_assert_ok(&s, Q_IMPERSONATE, Some(v)).await;
    let r = r.data.to_json()?;
    let ls_id = r.str("/orgImpersonate/inner/id");

    let imp = Impersonation::find()
        .filter(ImpersonationColumn::LoginSessionId.eq(ls_id))
        .one(&d.tmp.db)
        .await?;
    let Some(imp) = imp else {
        return TestErr::expect("Impersonation row should be created");
    };

    let q = "mutation t($id: ID!) { orgImpersonateRevoke(id: $id) { id } }";
    exec_assert_id(&s, q, &imp.id, &value!({ "orgImpersonateRevoke": { "id": imp.id } })).await;

    let ls_gone = LoginSession::find_by_id(ls_id).one(&d.tmp.db).await?;
    pretty_eq!(
        ls_gone.is_none(),
        true,
        "impersonated login session should be deleted after revoke"
    );

    let imp_after = Impersonation::find_by_id(&imp.id).one(&d.tmp.db).await?;
    let Some(imp_after) = imp_after else {
        return TestErr::expect("Impersonation row should still exist (soft deleted) after revoke");
    };
    pretty_eq!(
        imp_after.deleted_at.is_some(),
        true,
        "Impersonation row should be soft deleted after revoke"
    );

    d.tmp.drop().await
}
