use grand_line::prelude::*;

use crate::am_ctx::*;
use crate::models::*;

/// Creates a login session for user_id without setting a cookie (the secret is
/// returned to the caller, who uses it via the Authorization header instead, so
/// the admin's own cookie-based session is unaffected -- current_login_session
/// already checks the Authorization header before the cookie), and records an
/// Impersonation row via the ctx-aware exec so created_by_id (the current admin)
/// is tracked automatically.
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
