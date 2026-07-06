use crate::prelude::*;

pub trait AuthzOrg
where
    Self: EntityX + Send + Sync,
{
    fn authz_default_impl() -> Box<dyn AuthzOrgImpl> {
        Box::new(DefaultOrgImpl::<Self>(PhantomData))
    }
}
