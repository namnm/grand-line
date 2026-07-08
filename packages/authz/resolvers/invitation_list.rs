use crate::prelude::*;

/// A pending org invitation visible to the invited (authenticated) user, letting
/// them see it and decide whether to accept (org_invitation_resolve) or reject
/// (org_invitation_reject) it. email/secret/otp stay hidden, only the id (needed
/// to accept/reject) and the invitation's target org/role are exposed.
pub struct OrgInvitationView {
    pub id: String,
    pub org_id: String,
    pub role_id: String,
    pub will_expire_at: DateTimeUtc,
}
#[Object]
impl OrgInvitationView {
    async fn id(&self) -> &str {
        &self.id
    }
    async fn org_id(&self) -> &str {
        &self.org_id
    }
    async fn role_id(&self) -> &str {
        &self.role_id
    }
    async fn will_expire_at(&self) -> DateTimeUtc {
        self.will_expire_at
    }
}

/// Lists every pending org invitation addressed to the current authenticated
/// user's email.
pub async fn my_org_invitations_impl<U>(ctx: &Context<'_>) -> Res<Vec<OrgInvitationView>>
where
    U: AuthUser,
{
    ctx.auth_ensure_authenticated().await?;

    let tx = &*ctx.tx().await?;
    let user_id = ctx.auth().await?;
    let u = U::find().filter(U::col_id().eq(&user_id)).one_or_404(tx).await?;
    let email = U::get_email(&u);

    let invitations = Otp::find()
        .include_deleted(false)
        .filter(OtpColumn::Ty.eq(OTP_TY_ORG_INVITATION))
        .filter(OtpColumn::Email.eq(email))
        .all(tx)
        .await?;

    let c = ctx.auth_config();
    invitations
        .into_iter()
        .map(|inv| {
            let d = OtpDataOrgInvitation::from_json(inv.data)?;
            Ok(OrgInvitationView {
                id: inv.id,
                org_id: d.org_id,
                role_id: d.role_id,
                will_expire_at: inv.created_at + duration_ms(c.otp_expires_ms),
            })
        })
        .collect()
}
