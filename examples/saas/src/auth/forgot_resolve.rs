use grand_line::prelude::*;

use crate::models::*;

/// Resolves a forgot-type OTP, sets the new password for the user captured at request time,
/// and logs them in with a fresh session.
#[mutation]
fn forgot_resolve(data: OtpResolve, password: String) -> LoginSessionWithSecret {
    ensure_not_authenticated(ctx).await?;
    rand_utils::password_validate(&password)?;

    let t = otp_ensure_resolve(tx, OTP_TY_FORGOT, data).await?;
    let d = OtpDataForgot::from_json(t.data)?;

    let password_hashed = rand_utils::password_hash(&password)?;
    am_update!(User {
        id: d.user_id.clone(),
        password_hashed,
    })
    .exec_without_ctx(tx)
    .await?;

    let ls = login_session_create(ctx, tx, &d.user_id).await?;
    Otp::delete_by_id(&t.id).exec(tx).await?;

    ls
}
