use crate::prelude::*;

/// Resolves a Forgot-type OTP, sets the new password for the user captured at request time,
/// and logs them in with a fresh session.
pub async fn forgot_resolve_impl<U>(
    ctx: &Context<'_>,
    data: AuthOtpResolve,
    password: String,
) -> Res<LoginSessionWithSecret>
where
    U: AuthUser,
{
    ctx.auth_ensure_not_authenticated().await?;

    let tx = &*ctx.tx().await?;
    let h = &ctx.auth_config().handlers;

    h.password_validate(ctx, &password).await?;
    let lsd = ctx.login_session_data()?;

    let t = auth_otp_ensure_resolve(ctx, tx, AuthOtpTy::Forgot, data).await?;
    let d = AuthOtpDataForgot::from_json(t.data)?;

    let password_hashed = rand_utils::password_hash(&password)?;
    let mut am = U::A::defaults_on_update().set_id(&d.user_id);
    am.set(U::hashed_password_col(), password_hashed.into());
    let u = am.update(tx).await?;

    let ls = login_session_create(ctx, tx, &u.get_id(), &lsd).await?;
    AuthOtp::delete_by_id(t.id).exec(tx).await?;

    h.on_forgot_resolve(ctx, &d.user_id, &ls.inner).await?;

    Ok(ls)
}
