use crate::prelude::*;

static DEFAULT: LazyLock<AuthzConfig> = LazyLock::new(AuthzConfig::default);

/// Access to the AuthzConfig attached to the schema context.
pub trait AuthzConfigContext<'a>
where
    Self: CoreContext<'a>,
{
    /// Return the configured AuthzConfig, or the default if none was set on
    /// the schema context.
    fn authz_config(&self) -> &'a AuthzConfig {
        if let Some(cfg) = self.data_opt_impl::<AuthzConfig>() {
            cfg
        } else {
            &DEFAULT
        }
    }

    /// Return the error to raise when an authz check fails.
    fn authz_err(&self) -> &'a GrandLineErr {
        &self.authz_config().unauthorized_err
    }

    /// Return the AuthzOrgImpl registered on the schema context, or
    /// OrgImplNotFound if none was set.
    fn authz_org_impl(&self) -> Res<&'a dyn AuthzOrgImpl> {
        let r = self
            .data_opt_impl::<Box<dyn AuthzOrgImpl>>()
            .ok_or(MyErr::OrgImplNotFound)?
            .as_ref();
        Ok(r)
    }

    /// Return the AuthzRoleImpl registered on the schema context, or
    /// RoleImplNotFound if none was set.
    fn authz_role_impl(&self) -> Res<&'a dyn AuthzRoleImpl> {
        let r = self
            .data_opt_impl::<Box<dyn AuthzRoleImpl>>()
            .ok_or(MyErr::RoleImplNotFound)?
            .as_ref();
        Ok(r)
    }

    /// Return the AuthzCurrentUserImpl registered on the schema context, or
    /// CurrentUserImplNotFound if none was set.
    fn authz_current_user_impl(&self) -> Res<&'a dyn AuthzCurrentUserImpl> {
        let r = self
            .data_opt_impl::<Box<dyn AuthzCurrentUserImpl>>()
            .ok_or(MyErr::CurrentUserImplNotFound)?
            .as_ref();
        Ok(r)
    }
}

impl<'a> AuthzConfigContext<'a> for Context<'a> {
}
