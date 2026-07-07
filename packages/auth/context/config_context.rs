use crate::prelude::*;

static DEFAULT: LazyLock<AuthConfig> = LazyLock::new(AuthConfig::default);

/// Provides access to the configured AuthConfig, falling back to defaults when none is set.
pub trait AuthConfigContext<'a>
where
    Self: CoreContext<'a>,
{
    /// Returns the AuthConfig registered on the context, or the default config if none was set.
    fn auth_config(&self) -> &'a AuthConfig {
        if let Some(cfg) = self.data_opt_impl::<AuthConfig>() {
            cfg
        } else {
            &DEFAULT
        }
    }
}

impl<'a> AuthConfigContext<'a> for Context<'a> {
}
