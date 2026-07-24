use grand_line::prelude::*;

use crate::models::*;

#[gql_input]
pub struct InvitationCreate {
    pub email: Email,
    pub role_id: String,
}

/// Starts the org-invitation flow, creates an OTP_TY_ORG_INVITATION otp row and
/// returns it with its secret, the invited email must resolve it (as an
/// already-authenticated user, see orgInvitationResolve) to actually join the org.
#[mutation(authz(realm = "org"))]
fn invitation_create(data: InvitationCreate) -> OtpWithSecret {
    let org_id = ctx.authz().await?;

    otp_ensure_re_request(tx, OTP_TY_ORG_INVITATION, &data.email.0).await?;

    let otp = rand_utils::otp();
    let secret = rand_utils::secret();
    let (otp_salt, otp_hashed) = rand_utils::otp_hash(&otp)?;

    let t = am_create!(Otp {
        ty: OTP_TY_ORG_INVITATION.to_owned(),
        email: data.email.0,
        secret_hashed: rand_utils::secret_hash(&secret),
        data: OtpDataOrgInvitation {
            org_id,
            role_id: data.role_id,
        }
        .to_json()?,
        otp_salt,
        otp_hashed,
    })
    .exec_without_ctx(tx)
    .await?;

    // NOTE: replace this with a real mailer call.
    println!("send org invitation otp {otp} to {}", t.email);

    OtpWithSecret {
        inner: t,
        secret,
    }
}
