use crate::prelude::*;

pub trait AuthzOrg
where
    Self: EntityX + Send + Sync,
{
}
