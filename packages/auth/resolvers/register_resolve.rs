use crate::prelude::*;

/// Resolves a Register-type OTP, creates the user with the data captured at request time,
/// and logs them in with a fresh session.
pub async fn register_resolve_impl<U>(ctx: &Context<'_>, data: AuthOtpResolve) -> Res<LoginSessionWithSecret>
where
    U: AuthUser,
{
    ctx.auth_ensure_not_authenticated().await?;

    let tx = &*ctx.tx().await?;
    let lsd = ctx.login_session_data()?;
    let h = &ctx.auth_config().handlers;

    let t = auth_otp_ensure_resolve(ctx, tx, AuthOtpTy::Register, data).await?;
    let d = AuthOtpDataRegister::from_json(t.data)?;

    register_ensure_email_not_exists::<U>(tx, &t.email).await?;

    let mut am = U::A::defaults_on_create();
    am.set(U::email_col(), t.email.into());
    am.set(U::hashed_password_col(), d.password_hashed.into());
    let u = am.insert(tx).await?;

    let ls = login_session_create(ctx, tx, &u.get_id(), &lsd).await?;
    AuthOtp::delete_by_id(t.id).exec(tx).await?;

    h.on_register_resolve(ctx, &u.get_id(), &ls.inner).await?;

    Ok(ls)
}
