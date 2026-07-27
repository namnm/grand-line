use crate::prelude::*;

#[gql_input]
pub struct Forgot {
    pub email: Email,
}

#[mutation(auth(unauthenticated))]
fn forgot(data: Forgot) -> OtpWithSecret {
    ctx.auth_otp_ensure_re_request(OTP_TY_FORGOT, &data.email.0).await?;

    let u = User::find()
        .include_deleted(false)
        .filter(UserColumn::Email.eq(&data.email.0))
        .one_or_404(tx)
        .await?;

    let otp = rand_utils::otp();
    let secret = rand_utils::secret();
    let (otp_salt, otp_hashed) = rand_utils::otp_hash(&otp)?;

    let t = am_create!(Otp {
        ty: OTP_TY_FORGOT.to_owned(),
        email: data.email.0,
        secret_hashed: rand_utils::secret_hash(&secret),
        data: OtpDataForgot {
            user_id: u.id,
        }
        .to_json()?,
        otp_salt,
        otp_hashed,
    })
    .exec_without_ctx(tx)
    .await?;

    // NOTE: replace this with a real mailer call.
    println!("send forgot otp {otp} to {}", t.email);

    OtpWithSecret {
        inner: t,
        secret,
    }
}
