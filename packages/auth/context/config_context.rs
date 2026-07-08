use crate::prelude::*;

static DEFAULT: LazyLock<AuthConfig> = LazyLock::new(AuthConfig::default);

pub trait AuthConfigContext<'a> {
    fn auth_config(&self) -> &'a AuthConfig;
}

impl<'a> AuthConfigContext<'a> for Context<'a> {
    fn auth_config(&self) -> &'a AuthConfig {
        if let Some(cfg) = self.data_opt::<AuthConfig>() {
            cfg
        } else {
            &DEFAULT
        }
    }
}

pub trait AuthUserImplContext<'a> {
    fn auth_user_impl<U: AuthUser>(&self) -> Res<&'a AuthUserImpl<U>>;
}

impl<'a> AuthUserImplContext<'a> for Context<'a> {
    fn auth_user_impl<U: AuthUser>(&self) -> Res<&'a AuthUserImpl<U>> {
        self.data_opt::<AuthUserImpl<U>>()
            .ok_or_else(|| MyErr::UserImplNotFound.into())
    }
}
