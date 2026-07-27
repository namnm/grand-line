use super::prelude::*;

/// Aggregate of all core context traits, implement or bound against this for full access.
pub trait CoreContext<'a>
where
    Self: ImplContext<'a>
        + GrandLineDataContext<'a>
        + CacheContext<'a>
        + CoreConfigContext<'a>
        + TxContext<'a>
        + DataLoaderContext<'a>,
{
}

impl<'a> CoreContext<'a> for Context<'a> {
}
