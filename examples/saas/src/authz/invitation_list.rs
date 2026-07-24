use grand_line::prelude::*;

use crate::models::*;

/// A pending org invitation visible to the invited (authenticated) user, letting
/// them see it and decide whether to accept (orgInvitationResolve) or reject
/// (orgInvitationReject) it. email/secret/otp stay hidden, only the id (needed
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
#[query]
async fn my_org_invitations() -> Vec<OrgInvitationView> {
    let ls = ensure_authenticated(ctx).await?;
    let u = User::find_by_id(&ls.user_id).one_or_404(tx).await?;

    let invitations = Otp::find()
        .include_deleted(false)
        .filter(OtpColumn::Ty.eq(OTP_TY_ORG_INVITATION))
        .filter(OtpColumn::Email.eq(&u.email))
        .all(tx)
        .await?;

    invitations
        .into_iter()
        .map(|inv| {
            let d = OtpDataOrgInvitation::from_json(inv.data)?;
            Ok(OrgInvitationView {
                id: inv.id,
                org_id: d.org_id,
                role_id: d.role_id,
                will_expire_at: inv.created_at + duration_ms(OTP_EXPIRE_MS),
            })
        })
        .collect::<Res<Vec<_>>>()?
}
