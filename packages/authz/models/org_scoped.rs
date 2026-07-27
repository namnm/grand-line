use crate::prelude::*;

/// Marker trait for entities whose rows belong to a single org via an org_id column.
pub trait AuthzImplOrgId
where
    Self: EntityX,
{
    fn col_org_id() -> Self::C;
}
