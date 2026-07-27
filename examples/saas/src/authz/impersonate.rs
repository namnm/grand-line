use crate::prelude::*;

#[mutation(authz(realm = "org"))]
fn impersonate(user_id: String, reason: String) -> LoginSessionWithSecret {
    let org_id = ctx.authz().await?;

    UserInRole::find()
        .filter(UserInRoleColumn::UserId.eq(&user_id))
        .filter(UserInRoleColumn::OrgId.eq(&org_id))
        .exists_or_404(tx)
        .await?;

    let ip = ctx.get_ip()?;
    let ua = ctx.get_ua()?;

    let secret = rand_utils::secret();
    let ls = am_create!(LoginSession {
        user_id: user_id.clone(),
        secret_hashed: rand_utils::secret_hash(&secret),
        ip,
        ua: ua.to_json()?,
    })
    .exec_without_ctx(tx)
    .await?;

    am_create!(Impersonation {
        login_session_id: ls.id.clone(),
        user_id,
        org_id: Some(org_id),
        reason,
    })
    .exec(ctx)
    .await?;

    LoginSessionWithSecret {
        inner: ls,
        secret,
    }
}
