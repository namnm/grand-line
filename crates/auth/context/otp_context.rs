use crate::prelude::*;

/// Otp validation mechanics: attempt limiting, expiry, and re-request cooldown.
#[async_trait]
pub trait AuthOtpContext<'a>
where
    Self: AuthConfigContext<'a>,
{
    /// Consumes one resolve attempt on the matching otp row and validates the code and secret.
    async fn auth_otp_ensure_resolve(&self, ty: &str, id: &str, secret: &str, otp: &str) -> Res<AuthImplOtp>;

    /// Enforces the re-request cooldown for a given email/type, and deletes the
    /// stale otp row once the cooldown has passed so a fresh one can be created.
    async fn auth_otp_ensure_re_request(&self, ty: &str, email: &str) -> Res<()>;
}

#[async_trait]
impl<'a> AuthOtpContext<'a> for Context<'a> {
    async fn auth_otp_ensure_resolve(&self, ty: &str, id: &str, secret: &str, otp: &str) -> Res<AuthImplOtp> {
        let Some(m) = self.auth_otp_impl()?.increment(self, id, ty).await? else {
            return Err(MyErr::OtpResolveInvalid.into());
        };

        let c = self.auth_config();
        if !rand_utils::otp_eq(&m.otp_salt, &m.otp_hashed, otp)?
            || !rand_utils::secret_eq(&m.secret_hashed, secret)
            || m.total_attempt > c.otp_max_attempt
            || m.created_at + duration_ms(c.otp_expires_ms) < now()
        {
            return Err(MyErr::OtpResolveInvalid.into());
        }

        self.auth_otp_impl()?.reset(self, &m.id).await?;
        Ok(m)
    }

    async fn auth_otp_ensure_re_request(&self, ty: &str, email: &str) -> Res<()> {
        let Some(m) = self.auth_otp_impl()?.find(self, ty, email).await? else {
            return Ok(());
        };

        if m.created_at + duration_ms(self.auth_config().otp_re_request_ms) > now() {
            return Err(MyErr::OtpReRequestTooSoon.into());
        }

        self.auth_otp_impl()?.delete(self, ty, email).await?;
        Ok(())
    }
}
