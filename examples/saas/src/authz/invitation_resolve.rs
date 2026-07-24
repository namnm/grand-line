use grand_line::prelude::*;

use crate::err::SaasErr;
use crate::models::*;

/// Accepts a pending org invitation for the currently authenticated user, creating
/// a UserInRole for them. The otp's email must match the authenticated user's email.
/// Unlike the generic-plus-hook design this replaced, this is just a plain
/// mutation: User is a concrete type here, no <U: AuthUser> or handler trait
/// indirection needed.
#[mutation]
fn org_invitation_resolve(data: OtpResolve) -> UserInRoleGql {
    let ls = ensure_authenticated(ctx).await?;

    let t = otp_ensure_resolve(tx, OTP_TY_ORG_INVITATION, data).await?;

    let u = User::find_by_id(&ls.user_id).one_or_404(tx).await?;
    if !u.email.eq_ignore_ascii_case(&t.email) {
        return Err(SaasErr::InvitationEmailMismatch.into());
    }

    let d = OtpDataOrgInvitation::from_json(t.data)?;

    let uir = am_create!(UserInRole {
        user_id: ls.user_id,
        role_id: d.role_id,
        org_id: Some(d.org_id),
    })
    .exec_without_ctx(tx)
    .await?;

    Otp::delete_by_id(&t.id).exec(tx).await?;

    uir.into_gql(ctx).await?
}

/// Declines a pending org invitation. No authentication required (mirrors a
/// plain "unsubscribe"-style link): proof of ownership is the id+secret+otp
/// challenge itself, not a session.
#[mutation]
fn org_invitation_reject(data: OtpResolve) -> OtpGql {
    let t = otp_ensure_resolve(tx, OTP_TY_ORG_INVITATION, data).await?;
    Otp::delete_by_id(&t.id).exec(tx).await?;
    OtpGql::from_id(&t.id)
}
