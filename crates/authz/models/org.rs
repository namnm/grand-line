use crate::prelude::*;

/// Marker trait for entities that can act as the org lookup target of
/// AuthzHttpContext.
pub trait AuthzOrg
where
    Self: EntityX + Send + Sync,
{
    /// Build the default AuthzOrgImpl for this entity, used unless a custom
    /// implementation is registered.
    fn authz_default_impl() -> Box<dyn AuthzOrgImpl> {
        Box::new(DefaultOrgImpl::<Self>(PhantomData))
    }
}
