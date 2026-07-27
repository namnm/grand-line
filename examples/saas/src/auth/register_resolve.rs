use crate::prelude::*;

#[mutation(auth(unauthenticated))]
fn register_resolve(data: OtpResolve) -> LoginSessionWithSecret {
    let m = ctx
        .auth_otp_ensure_resolve(OTP_TY_REGISTER, &data.id, &data.secret, &data.otp)
        .await?;
    let d = OtpDataRegister::from_json(m.data)?;

    let u = am_create!(User {
        email: m.email.clone(),
        password_hashed: d.password_hashed,
    })
    .exec_without_ctx(tx)
    .await?;

    let ls = login_session_create(ctx, tx, &u.id).await?;
    Otp::delete_by_id(&m.id).exec(tx).await?;

    ls
}
