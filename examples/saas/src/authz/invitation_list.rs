use crate::prelude::*;

#[derive(SimpleObject)]
pub struct OrgInvitation {
    pub id: String,
    pub org_id: String,
    pub role_id: String,
    pub will_expire_at: DateTimeUtc,
}

#[query(auth)]
async fn my_org_invitations() -> Vec<OrgInvitation> {
    let user_id = ctx.auth().await?;
    let u = User::find_by_id(&user_id).one_or_404(tx).await?;

    let invitations = Otp::find()
        .include_deleted(false)
        .filter(OtpColumn::Ty.eq(OTP_TY_ORG_INVITATION))
        .filter(OtpColumn::Email.eq(&u.email))
        .all(tx)
        .await?;

    invitations
        .into_iter()
        .map(|i| {
            let d = OtpDataOrgInvitation::from_json(i.data)?;
            Ok(OrgInvitation {
                id: i.id,
                org_id: d.org_id,
                role_id: d.role_id,
                will_expire_at: i.created_at + duration_ms(ctx.auth_config().otp_expires_ms),
            })
        })
        .collect::<Res<Vec<_>>>()?
}
