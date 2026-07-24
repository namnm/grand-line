use grand_line::prelude::*;

use crate::err::SaasErr;

pub const OTP_MAX_ATTEMPT: i64 = 5;
pub const OTP_EXPIRE_MS: i64 = 10 * 60 * 1000;
pub const OTP_RE_REQUEST_MS: i64 = 60 * 1000;

pub const OTP_TY_REGISTER: &str = "register";
pub const OTP_TY_FORGOT: &str = "forgot";
pub const OTP_TY_ORG_INVITATION: &str = "org_invitation";

/// A purpose-tagged one-time code, ty identifies the flow it belongs to (see
/// the OTP_TY_* constants above).
#[model(updated_at = false, deleted_at = false, by_id = false)]
pub struct Otp {
    pub email: String,

    #[graphql(skip)]
    pub ty: String,

    /// Hash of the opaque secret returned to the client with the row id, checked
    /// alongside the OTP code so the resolve endpoint cannot be guessed by id alone.
    #[graphql(skip)]
    pub secret_hashed: String,

    /// Salt and hash of the one-time password code delivered to the user, e.g. by email.
    #[graphql(skip)]
    pub otp_salt: String,
    #[graphql(skip)]
    pub otp_hashed: String,

    /// Type-specific payload, see OtpDataRegister/OtpDataForgot/OtpDataOrgInvitation.
    #[graphql(skip)]
    pub data: JsonValue,

    #[default(0)]
    #[graphql(skip)]
    pub total_attempt: i64,
    #[resolver(sql_dep = "total_attempt")]
    pub remaining_attempt: i64,

    #[resolver(sql_dep = "created_at")]
    pub will_expire_at: DateTimeUtc,
    #[resolver(sql_dep = "created_at")]
    pub can_re_request_at: DateTimeUtc,
}

async fn resolve_remaining_attempt(o: &OtpGql, _ctx: &Context<'_>) -> Res<i64> {
    let t = o.total_attempt.ok_or(CoreDbErr::GqlResolverNone)?;
    Ok(OTP_MAX_ATTEMPT - t)
}
async fn resolve_will_expire_at(o: &OtpGql, _ctx: &Context<'_>) -> Res<DateTimeUtc> {
    let t = o.created_at.ok_or(CoreDbErr::GqlResolverNone)?;
    Ok(t + duration_ms(OTP_EXPIRE_MS))
}
async fn resolve_can_re_request_at(o: &OtpGql, _ctx: &Context<'_>) -> Res<DateTimeUtc> {
    let t = o.created_at.ok_or(CoreDbErr::GqlResolverNone)?;
    Ok(t + duration_ms(OTP_RE_REQUEST_MS))
}

/// Payload stored in Otp.data for an OTP_TY_REGISTER row.
#[derive(Serialize, Deserialize)]
pub struct OtpDataRegister {
    pub password_hashed: String,
}
/// Payload stored in Otp.data for an OTP_TY_FORGOT row.
#[derive(Serialize, Deserialize)]
pub struct OtpDataForgot {
    pub user_id: String,
}
/// Payload stored in Otp.data for an OTP_TY_ORG_INVITATION row.
#[derive(Serialize, Deserialize)]
pub struct OtpDataOrgInvitation {
    pub org_id: String,
    pub role_id: String,
}

/// To only expose secret in some operations, not the others.
pub struct OtpWithSecret {
    pub inner: OtpSql,
    pub secret: String,
}
#[Object]
impl OtpWithSecret {
    pub async fn secret(&self) -> String {
        self.secret.clone()
    }
    pub async fn inner(&self, ctx: &Context<'_>) -> Res<OtpGql> {
        let r = self.inner.clone().into_gql(ctx).await?;
        Ok(r)
    }
}

#[gql_input]
pub struct OtpResolve {
    pub id: String,
    pub secret: String,
    pub otp: String,
}

/// Consumes one resolve attempt on the matching OTP row and validates the code and secret.
/// Returns SaasErr::OtpResolveInvalid if the id/type does not exist, the code or secret does
/// not match, the attempt count is exceeded, or the OTP has expired. On success, resets the
/// attempt counter to 0.
pub async fn otp_ensure_resolve(tx: &DatabaseTransaction, ty: &str, data: OtpResolve) -> Res<OtpSql> {
    let u = Otp::update_many()
        .include_deleted(false)
        .filter_by_id(&data.id)
        .filter(OtpColumn::Ty.eq(ty))
        .set(OtpActiveModel::defaults_on_update())
        .col_expr(
            OtpColumn::TotalAttempt,
            Expr::col(OtpColumn::TotalAttempt).add(1),
        );

    if u.exec(tx).await?.rows_affected == 0 {
        Err(SaasErr::OtpResolveInvalid)?;
    }
    let t = Otp::find()
        .include_deleted(false)
        .filter_by_id(&data.id)
        .one(tx)
        .await?
        .ok_or(SaasErr::OtpResolveInvalid)?;

    if !rand_utils::otp_eq(&t.otp_salt, &t.otp_hashed, &data.otp)?
        || !rand_utils::secret_eq(&t.secret_hashed, &data.secret)
        || t.total_attempt > OTP_MAX_ATTEMPT
        || t.created_at + duration_ms(OTP_EXPIRE_MS) < now()
    {
        return Err(SaasErr::OtpResolveInvalid.into());
    }

    let t = am_update!(Otp {
        total_attempt: 0,
        ..t.into_active_model()
    })
    .exec_without_ctx(tx)
    .await?;

    Ok(t)
}

/// Enforces the re-request cooldown for a given email/type, and deletes the stale OTP row
/// once the cooldown has passed so a fresh one can be created. No-op if no row exists.
pub async fn otp_ensure_re_request(tx: &DatabaseTransaction, ty: &str, email: &str) -> Res<()> {
    let t = Otp::find()
        .include_deleted(false)
        .filter(OtpColumn::Ty.eq(ty))
        .filter(OtpColumn::Email.eq(email))
        .one(tx)
        .await?;
    let Some(t) = t else {
        return Ok(());
    };

    if t.created_at + duration_ms(OTP_RE_REQUEST_MS) > now() {
        return Err(SaasErr::OtpReRequestTooSoon.into());
    }

    Otp::delete_many()
        .filter(OtpColumn::Ty.eq(ty))
        .filter(OtpColumn::Email.eq(email))
        .exec(tx)
        .await?;

    Ok(())
}
