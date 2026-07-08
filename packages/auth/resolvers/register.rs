use crate::prelude::*;

#[gql_input]
pub struct Register {
    pub email: Email,
    pub password: String,
}

/// Starts registration, creates a Register-type OTP row and returns it with its secret,
/// the caller must resolve it with the OTP code to actually create the user.
pub async fn register_impl<U>(ctx: &Context<'_>, data: Register) -> Res<OtpWithSecret>
where
    U: AuthUser,
{
    ctx.auth_ensure_not_authenticated().await?;

    let tx = &*ctx.tx().await?;
    let h = &ctx.auth_config().handlers;

    register_ensure_email_not_exists::<U>(tx, &data.email.0).await?;
    otp_ensure_re_request(ctx, tx, OTP_TY_REGISTER, &data.email.0).await?;
    h.password_validate(ctx, &data.password).await?;

    let otp = h.otp(ctx).await?;
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

    h.on_otp_create(ctx, &t, &otp).await?;

    Ok(OtpWithSecret {
        inner: t,
        secret,
    })
}

pub async fn register_ensure_email_not_exists<U>(tx: &DatabaseTransaction, email: &str) -> Res<()>
where
    U: AuthUser,
{
    let exists = U::find()
        .include_deleted(false)
        .filter(U::email_col().eq(email))
        .exists(tx)
        .await?;
    if exists {
        return Err(MyErr::RegisterEmailExists.into());
    }
    Ok(())
}
