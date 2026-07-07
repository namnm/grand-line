use crate::prelude::*;

#[gql_input]
pub struct AuthOtpResolve {
    pub id: String,
    pub secret: String,
    pub otp: String,
}

#[mutation(auth(unauthenticated))]
fn auth_otp_resolve(ty: AuthOtpTy, data: AuthOtpResolve) -> AuthOtpGql {
    ctx.auth_ensure_not_authenticated().await?;

    let tx = &*ctx.tx().await?;
    auth_otp_ensure_resolve(ctx, tx, ty, data).await?.into_gql(ctx).await?
}

/// Consumes one resolve attempt on the matching OTP row and validates the code and secret.
/// Returns MyErr::OtpResolveInvalid if the id/type does not exist, the code or secret does
/// not match, the attempt count is exceeded, or the OTP has expired. On success, resets the
/// attempt counter to 0.
pub async fn auth_otp_ensure_resolve(
    ctx: &Context<'_>,
    tx: &DatabaseTransaction,
    ty: AuthOtpTy,
    data: AuthOtpResolve,
) -> Res<AuthOtpSql> {
    let u = AuthOtp::update_many()
        .include_deleted(false)
        .filter_by_id(&data.id)
        .filter(AuthOtpColumn::Ty.eq(ty))
        .set(AuthOtpActiveModel::defaults_on_update())
        .col_expr(
            AuthOtpColumn::TotalAttempt,
            Expr::col(AuthOtpColumn::TotalAttempt).add(1),
        );

    #[cfg(feature = "postgres")]
    let t = {
        u.exec_with_returning(tx)
            .await?
            .first()
            .ok_or(MyErr::OtpResolveInvalid)?
            .to_owned()
    };
    #[cfg(not(feature = "postgres"))]
    let t = {
        if u.exec(tx).await?.rows_affected == 0 {
            Err(MyErr::OtpResolveInvalid)?;
        }
        AuthOtp::find()
            .include_deleted(false)
            .filter_by_id(&data.id)
            .one(tx)
            .await?
            .ok_or(MyErr::OtpResolveInvalid)?
    };

    let c = ctx.auth_config();
    if !rand_utils::otp_eq(&t.otp_salt, &t.otp_hashed, &data.otp)?
        || !rand_utils::secret_eq(&t.secret_hashed, &data.secret)
        || t.total_attempt > c.otp_max_attempt
        || t.created_at + duration_ms(c.otp_expires_ms) < now()
    {
        return Err(MyErr::OtpResolveInvalid.into());
    }

    let t = am_update!(AuthOtp {
        total_attempt: 0,
        ..t.into_active_model()
    })
    .exec_without_ctx(tx)
    .await?;

    Ok(t)
}

/// Enforces the re-request cooldown for a given email/type, and deletes the stale OTP row
/// once the cooldown has passed so a fresh one can be created. No-op if no row exists.
pub async fn auth_otp_ensure_re_request(
    ctx: &Context<'_>,
    tx: &DatabaseTransaction,
    ty: AuthOtpTy,
    email: &str,
) -> Res<()> {
    let t = AuthOtp::find()
        .include_deleted(false)
        .filter(AuthOtpColumn::Ty.eq(ty))
        .filter(AuthOtpColumn::Email.eq(email))
        .one(tx)
        .await?;
    let Some(t) = t else {
        return Ok(());
    };

    let c = ctx.auth_config();
    if t.created_at + duration_ms(c.otp_re_request_ms) > now() {
        return Err(MyErr::OtpReRequestTooSoon.into());
    }

    AuthOtp::delete_many()
        .filter(AuthOtpColumn::Ty.eq(ty))
        .filter(AuthOtpColumn::Email.eq(email))
        .exec(tx)
        .await?;

    Ok(())
}
