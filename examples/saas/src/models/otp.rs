use crate::prelude::*;

pub const OTP_TY_REGISTER: &str = "register";
pub const OTP_TY_FORGOT: &str = "forgot";
pub const OTP_TY_ORG_INVITATION: &str = "org_invitation";

/// A purpose-tagged one-time code, ty identifies the flow it belongs to.
#[model(updated_at = false, deleted_at = false, by_id = false)]
#[auth_otp]
pub struct Otp {
    #[graphql(skip)]
    pub ty: String,
    pub email: String,

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

async fn resolve_remaining_attempt(o: &OtpGql, ctx: &Context<'_>) -> Res<i64> {
    let t = o.total_attempt.ok_or(CoreDbErr::GqlResolverNone)?;
    Ok(ctx.auth_config().otp_max_attempt - t)
}
async fn resolve_will_expire_at(o: &OtpGql, ctx: &Context<'_>) -> Res<DateTimeUtc> {
    let t = o.created_at.ok_or(CoreDbErr::GqlResolverNone)?;
    Ok(t + duration_ms(ctx.auth_config().otp_expires_ms))
}
async fn resolve_can_re_request_at(o: &OtpGql, ctx: &Context<'_>) -> Res<DateTimeUtc> {
    let t = o.created_at.ok_or(CoreDbErr::GqlResolverNone)?;
    Ok(t + duration_ms(ctx.auth_config().otp_re_request_ms))
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
    pub async fn id(&self) -> String {
        self.inner.id.clone()
    }
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
