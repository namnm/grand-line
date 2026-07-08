use crate::prelude::*;

pub async fn forgot_resolve_impl<U: AuthUser>(
    ctx: &Context<'_>,
    data: AuthOtpResolve,
    password: String,
) -> Res<LoginSessionWithSecret> {
    let tx = &*ctx.tx().await?;
    let h = &ctx.auth_config().handlers;
    let ih = &ctx.auth_user_impl::<U>()?.handlers;

    h.password_validate(ctx, &password).await?;
    let lsd = login_session_data(ctx)?;

    let t = auth_otp_ensure_resolve(ctx, tx, AuthOtpTy::Forgot, data).await?;
    let d = AuthOtpDataForgot::from_json(t.data)?;

    let password_hashed = rand_utils::password_hash(&password)?;
    let mut am = U::A::defaults_on_update().set_id(&d.user_id);
    am.set(U::password_col(), password_hashed.into());
    let u = am.update(tx).await?;

    let ls = login_session_create(ctx, tx, &u.get_id(), &lsd).await?;
    AuthOtp::delete_by_id(t.id).exec(tx).await?;

    ih.on_forgot_resolve(ctx, &u, &ls.inner).await?;

    Ok(ls)
}
