use crate::prelude::*;

// ---------------------------------------------------------------------------
// OTP model
// ---------------------------------------------------------------------------

/// A purpose-tagged one-time code, ty identifies the flow it belongs to (see
/// OTP_TY_REGISTER, OTP_TY_FORGOT, and any ty a downstream package defines for
/// its own flow, e.g. authz's org invitation).
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

    /// Type-specific payload, see OtpDataRegister and OtpDataForgot.
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

// ---------------------------------------------------------------------------
// Type-specific OTP payloads
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// GraphQL resolver fields for OTP
// ---------------------------------------------------------------------------

async fn resolve_remaining_attempt(o: &OtpGql, ctx: &Context<'_>) -> Res<i64> {
    let c = ctx.auth_config();
    let t = o.total_attempt.ok_or(CoreDbErr::GqlResolverNone)?;
    let m = c.otp_max_attempt;
    Ok(m - t)
}
async fn resolve_will_expire_at(o: &OtpGql, ctx: &Context<'_>) -> Res<DateTimeUtc> {
    let c = ctx.auth_config();
    let t = o.created_at.ok_or(CoreDbErr::GqlResolverNone)?;
    let d = duration_ms(c.otp_expires_ms);
    Ok(t + d)
}
async fn resolve_can_re_request_at(o: &OtpGql, ctx: &Context<'_>) -> Res<DateTimeUtc> {
    let c = ctx.auth_config();
    let t = o.created_at.ok_or(CoreDbErr::GqlResolverNone)?;
    let d = duration_ms(c.otp_re_request_ms);
    Ok(t + d)
}

// ---------------------------------------------------------------------------
// OTP with secret exposed
// ---------------------------------------------------------------------------

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
