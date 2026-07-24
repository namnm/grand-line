use grand_line::prelude::*;

use crate::models::*;

/// Resolves every pending org invitation matching email into a UserInRole for
/// user_id, deleting each consumed Otp row. Called directly from
/// auth::register_resolve so a brand-new signup is immediately joined to any
/// org they were already invited to -- this is the whole point of moving
/// auth+authz into application code: no hook, no generic wiring, just a plain
/// function call inline where it's needed.
pub async fn org_invitation_resolve_by_email(tx: &DatabaseTransaction, user_id: &str, email: &str) -> Res<()> {
    let invitations = Otp::find()
        .include_deleted(false)
        .filter(OtpColumn::Ty.eq(OTP_TY_ORG_INVITATION))
        .filter(OtpColumn::Email.eq(email))
        .all(tx)
        .await?;

    for inv in invitations {
        let d = OtpDataOrgInvitation::from_json(inv.data.clone())?;
        am_create!(UserInRole {
            user_id: user_id.to_owned(),
            role_id: d.role_id,
            org_id: Some(d.org_id),
        })
        .exec_without_ctx(tx)
        .await?;
        Otp::delete_by_id(&inv.id).exec(tx).await?;
    }

    Ok(())
}
