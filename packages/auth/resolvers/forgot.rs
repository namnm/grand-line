use crate::prelude::*;

#[gql_input]
pub struct Forgot {
    pub email: Email,
}

/// Starts the forgot-password flow, creates a Forgot-type OTP row for the user with this
/// email and returns it with its secret, the caller must resolve it to set a new password.
/// Errors with a 404 if no user has this email.
pub async fn forgot_impl<U>(ctx: &Context<'_>, data: Forgot) -> Res<OtpWithSecret>
where
    U: AuthUser,
{
    ctx.auth_ensure_not_authenticated().await?;

    let tx = &*ctx.tx().await?;
    let h = &ctx.auth_config().handlers;

    otp_ensure_re_request(ctx, tx, OTP_TY_FORGOT, &data.email.0).await?;

    let u = U::find()
        .include_deleted(false)
        .filter(U::email_col().eq(&data.email.0))
        .one_or_404(tx)
        .await?;

    let otp = h.otp(ctx).await?;
    let secret = rand_utils::secret();
    let (otp_salt, otp_hashed) = rand_utils::otp_hash(&otp)?;

    let t = am_create!(Otp {
        ty: OTP_TY_FORGOT.to_owned(),
        email: data.email.0,
        secret_hashed: rand_utils::secret_hash(&secret),
        data: OtpDataForgot {
            user_id: u.get_id(),
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
