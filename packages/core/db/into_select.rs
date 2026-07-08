use super::prelude::*;

/// Helper trait to create `sea_orm` Select from types like Filter, `OrderBy`...
pub trait IntoSelect<E>
where
    E: EntityX,
{
    /// Helper to create `sea_orm` Select from types like Filter, `OrderBy`...
    fn into_select(self) -> Select<E>;

    /// Shortcut for `self.into_select().gql_select(ctx)`.
    fn gql_select(self, ctx: &Context<'_>) -> Res<Selector<SelectModel<E::G>>>;

    /// Shortcut for `self.into_select().gql_select_id()`.
    fn gql_select_id(self) -> Selector<SelectModel<E::G>>;
}

/// Automatically implement `IntoSelect` for `ChainSelect`.
impl<E, C> IntoSelect<E> for C
where
    E: EntityX,
    C: ChainSelect<E>,
{
    fn into_select(self) -> Select<E> {
        self.chain_select(E::find())
    }
    fn gql_select(self, ctx: &Context<'_>) -> Res<Selector<SelectModel<E::G>>> {
        self.into_select().gql_select(ctx)
    }
    fn gql_select_id(self) -> Selector<SelectModel<E::G>> {
        self.into_select().gql_select_id()
    }
}
