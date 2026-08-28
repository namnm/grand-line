use crate::prelude::*;

#[mutation(check = unauthenticated)]
fn forgot_resolve(data: OtpResolve, password: String) -> LoginSessionWithSecret {
    rand_utils::password_validate(&password)?;

    let m = ctx
        .auth_otp_ensure_resolve(OTP_TY_FORGOT, &data.id, &data.secret, &data.otp)
        .await?;
    let d = OtpDataForgot::from_json(m.data)?;

    let password_hashed = rand_utils::password_hash(&password)?;
    am_update!(User {
        id: d.user_id.clone(),
        password_hashed,
    })
    .exec_without_ctx(db)
    .await?;

    let ls = login_session_create(ctx, db, &d.user_id).await?;
    Otp::delete_by_id(&m.id).exec(db).await?;

    ls
}
