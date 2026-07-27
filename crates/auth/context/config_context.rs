use crate::prelude::*;

static DEFAULT: LazyLock<AuthConfig> = LazyLock::new(AuthConfig::default);

/// Access to the AuthConfig and DI implementations attached to the schema context.
pub trait AuthConfigContext<'a>
where
    Self: CoreContext<'a>,
{
    /// Return the configured AuthConfig, or the default if none was set on the schema context.
    fn auth_config(&self) -> &'a AuthConfig {
        if let Some(cfg) = self.data_opt_impl::<AuthConfig>() {
            cfg
        } else {
            &DEFAULT
        }
    }

    /// Return the AuthSessionImpl registered on the schema context.
    fn auth_session_impl(&self) -> Res<&'a dyn AuthSessionImpl> {
        let r = self
            .data_opt_impl::<Box<dyn AuthSessionImpl>>()
            .ok_or(MyErr::SessionImplNotFound)?
            .as_ref();
        Ok(r)
    }

    /// Return the AuthOtpImpl registered on the schema context.
    fn auth_otp_impl(&self) -> Res<&'a dyn AuthOtpImpl> {
        let r = self
            .data_opt_impl::<Box<dyn AuthOtpImpl>>()
            .ok_or(MyErr::OtpImplNotFound)?
            .as_ref();
        Ok(r)
    }
}

impl<'a> AuthConfigContext<'a> for Context<'a> {
}
