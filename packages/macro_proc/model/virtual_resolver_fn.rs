use crate::prelude::*;

/// A ResolverFn whose GQL_SELECT entry also needs to know which SQL columns it
/// reads, so the field can be loaded even when it is not selected by its own name.
pub trait VirtualResolverFn
where
    Self: ResolverFn,
{
    /// SQL column names this resolver depends on.
    fn sql_dep(&self) -> SynRes<Vec<String>>;
}
