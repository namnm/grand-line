use crate::prelude::*;

#[gql_input]
pub struct Register {
    pub email: Email,
    pub password: String,
}

#[mutation(auth(unauthenticated))]
fn register(data: Register) -> OtpWithSecret {
    let exists = User::find()
        .include_deleted(false)
        .filter(UserColumn::Email.eq(&data.email.0))
        .exists(tx)
        .await?;
    if exists {
        return Err(SaasErr::RegisterEmailExists.into());
    }
    ctx.auth_otp_ensure_re_request(OTP_TY_REGISTER, &data.email.0).await?;
    rand_utils::password_validate(&data.password)?;

    let otp = rand_utils::otp();
    let (otp_salt, otp_hashed) = rand_utils::otp_hash(&otp)?;
    let secret = rand_utils::secret();

    let t = am_create!(Otp {
        ty: OTP_TY_REGISTER.to_owned(),
        email: data.email.0,
        secret_hashed: rand_utils::secret_hash(&secret),
        data: OtpDataRegister {
            password_hashed: rand_utils::password_hash(&data.password)?,
        }
        .to_json()?,
        otp_salt,
        otp_hashed,
    })
    .exec_without_ctx(tx)
    .await?;

    // NOTE: replace this with a real mailer call.
    println!("send register otp {otp} to {}", t.email);

    OtpWithSecret {
        inner: t,
        secret,
    }
}
