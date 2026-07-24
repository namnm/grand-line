use grand_line::prelude::*;

use crate::authz::org_invitation_resolve_by_email;
use crate::models::*;

/// Resolves a register-type OTP, creates the user with the data captured at request time,
/// and logs them in with a fresh session.
#[mutation]
fn register_resolve(data: OtpResolve) -> LoginSessionWithSecret {
    ensure_not_authenticated(ctx).await?;

    let t = otp_ensure_resolve(tx, OTP_TY_REGISTER, data).await?;
    let d = OtpDataRegister::from_json(t.data)?;

    let u = am_create!(User {
        email: t.email.clone(),
        password_hashed: d.password_hashed,
    })
    .exec_without_ctx(tx)
    .await?;

    // Chains straight into any orgs this email was already invited to, no hook
    // or generic wiring needed since register_resolve owns the concrete User
    // it just created.
    org_invitation_resolve_by_email(tx, &u.id, &t.email).await?;

    let ls = login_session_create(ctx, tx, &u.id).await?;
    Otp::delete_by_id(&t.id).exec(tx).await?;

    ls
}
