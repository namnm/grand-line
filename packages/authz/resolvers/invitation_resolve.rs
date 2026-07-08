use crate::prelude::*;

#[mutation]
fn org_invitation_reject(data: OtpResolve) -> OtpGql {
    let tx = &*ctx.tx().await?;
    let h = &ctx.authz_config().handlers;

    let t = otp_ensure_resolve(ctx, tx, OTP_TY_ORG_INVITATION, data).await?;
    Otp::delete_by_id(&t.id).exec(tx).await?;

    h.on_org_invitation_reject(ctx, &t).await?;

    OtpGql::from_id(&t.id)
}
