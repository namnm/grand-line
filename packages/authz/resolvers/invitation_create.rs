use crate::prelude::*;

/// Payload stored in Otp.data for an OTP_TY_ORG_INVITATION row.
#[derive(Serialize, Deserialize)]
pub struct OtpDataOrgInvitation {
    pub org_id: String,
    pub role_id: String,
}

#[gql_input]
pub struct OrgInvitationCreate {
    pub email: Email,
    pub role_id: String,
}

#[mutation(authz(realm = "org"))]
fn org_invitation_create(data: OrgInvitationCreate) -> OtpWithSecret {
    let org_id = ctx.authz().await?;
    invitation_create_impl(ctx, data.email, org_id, data.role_id).await?
}

#[gql_input]
pub struct SystemInvitationCreate {
    pub email: Email,
    pub role_id: String,
    pub org_id: String,
}

#[mutation(authz(realm = "system", skip_org))]
fn system_invitation_create(data: SystemInvitationCreate) -> OtpWithSecret {
    invitation_create_impl(ctx, data.email, data.org_id, data.role_id).await?
}

/// Starts the org-invitation flow, creates an OTP_TY_ORG_INVITATION otp row and
/// returns it with its secret, the invited email must resolve it (as an
/// already-authenticated user) to actually join the org.
async fn invitation_create_impl(
    ctx: &Context<'_>,
    email: Email,
    org_id: String,
    role_id: String,
) -> Res<OtpWithSecret> {
    let tx = &*ctx.tx().await?;
    let h = &ctx.auth_config().handlers;

    otp_ensure_re_request(ctx, tx, OTP_TY_ORG_INVITATION, &email.0).await?;

    let otp = h.otp(ctx).await?;
    let secret = rand_utils::secret();
    let (otp_salt, otp_hashed) = rand_utils::otp_hash(&otp)?;

    let t = am_create!(Otp {
        ty: OTP_TY_ORG_INVITATION.to_owned(),
        email: email.0,
        secret_hashed: rand_utils::secret_hash(&secret),
        data: OtpDataOrgInvitation {
            org_id,
            role_id,
        }
        .to_json()?,
        otp_salt,
        otp_hashed,
    })
    .exec_without_ctx(tx)
    .await?;

    h.on_otp_create(ctx, &t, &otp).await?;

    Ok(OtpWithSecret {
        inner: t,
        secret,
    })
}
