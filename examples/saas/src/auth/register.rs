use grand_line::prelude::*;

use crate::err::SaasErr;
use crate::models::*;

#[gql_input]
pub struct Register {
    pub email: Email,
    pub password: String,
}

/// Starts registration, creates a register-type OTP row and returns it with its secret,
/// the caller must resolve it with the OTP code to actually create the user.
#[mutation]
fn register(data: Register) -> OtpWithSecret {
    ensure_not_authenticated(ctx).await?;

    let exists = User::find()
        .include_deleted(false)
        .filter(UserColumn::Email.eq(&data.email.0))
        .exists(tx)
        .await?;
    if exists {
        return Err(SaasErr::RegisterEmailExists.into());
    }
    otp_ensure_re_request(tx, OTP_TY_REGISTER, &data.email.0).await?;
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

    // NOTE: replace this with a real mailer call, this is a stand-in like
    // examples/simple_todo's println! calls.
    println!("send register otp {otp} to {}", t.email);

    OtpWithSecret {
        inner: t,
        secret,
    }
}
