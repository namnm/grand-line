use crate::prelude::*;

#[mutation(auth)]
fn org_invitation_resolve(data: OtpResolve) -> UserInRoleGql {
    let user_id = ctx.auth().await?;

    let m = ctx
        .auth_otp_ensure_resolve(OTP_TY_ORG_INVITATION, &data.id, &data.secret, &data.otp)
        .await?;

    let u = User::find_by_id(&user_id).one_or_404(tx).await?;
    if u.email != m.email {
        return Err(SaasErr::InvitationEmailMismatch.into());
    }

    let d = OtpDataOrgInvitation::from_json(m.data)?;

    let uir = am_create!(UserInRole {
        user_id,
        role_id: d.role_id,
        org_id: Some(d.org_id),
    })
    .exec_without_ctx(tx)
    .await?;

    Otp::delete_by_id(&m.id).exec(tx).await?;

    uir.into_gql(ctx).await?
}

#[mutation(auth)]
fn org_invitation_reject(data: OtpResolve) -> OtpGql {
    // No authentication required (mirrors a plain "unsubscribe"-style link).
    // Proof of ownership is the id+secret+otp challenge itself, not a session.
    let m = ctx
        .auth_otp_ensure_resolve(OTP_TY_ORG_INVITATION, &data.id, &data.secret, &data.otp)
        .await?;
    Otp::delete_by_id(&m.id).exec(tx).await?;
    OtpGql::from_id(&m.id)
}
