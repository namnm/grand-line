use super::prelude::*;

/// Helper trait to chain sea_orm Select of different types like FilterX, OrderBy, Pagination..
pub trait ChainSelect<E>
where
    E: EntityX,
{
    /// Helper to chain sea_orm Select of different types like FilterX, OrderBy, Pagination..
    fn chain_select(self, q: Select<E>) -> Select<E>;
}

/// Automatically implement ChainSelect for Option<ChainSelect>.
impl<E, C> ChainSelect<E> for Option<C>
where
    E: EntityX,
    C: ChainSelect<E>,
{
    fn chain_select(self, q: Select<E>) -> Select<E> {
        match self {
            Some(c) => c.chain_select(q),
            None => q,
        }
    }
}

/// Automatically implement ChainSelect for Vec<ChainSelect>.
impl<E, C> ChainSelect<E> for Vec<C>
where
    E: EntityX,
    C: ChainSelect<E>,
{
    fn chain_select(self, mut q: Select<E>) -> Select<E> {
        for c in self {
            q = c.chain_select(q);
        }
        q
    }
}
