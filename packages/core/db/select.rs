use super::prelude::*;

/// Abstract extra Select methods implementation.
#[async_trait]
pub trait SelectX<E>
where
    E: EntityX,
{
    /// Helper to chain with ChainSelect.
    fn chain<C>(self, c: C) -> Self
    where
        C: ChainSelect<E>;

    /// Select only columns from requested fields in the graphql context.
    fn gql_select_with_look_ahead(self, look_ahead: &[LookaheadX<E>], col: E::C) -> Res<Selector<SelectModel<E::G>>>;
    /// Select only columns from requested fields in the graphql context.
    fn gql_select(self, ctx: &Context<'_>) -> Res<Selector<SelectModel<E::G>>>;

    /// Select only id for the graphql delete response.
    fn gql_select_id(self) -> Selector<SelectModel<E::G>>;

    /// Helper to check if exists and return error if not.
    async fn exists_or_404<D>(self, tx: &D) -> Res<()>
    where
        D: ConnectionTrait;
}

/// Automatically implement for Select<E>.
#[async_trait]
impl<E> SelectX<E> for Select<E>
where
    E: EntityX,
{
    fn chain<C>(self, c: C) -> Self
    where
        C: ChainSelect<E>,
    {
        c.chain_select(self)
    }

    fn gql_select_with_look_ahead(self, look_ahead: &[LookaheadX<E>], col: E::C) -> Res<Selector<SelectModel<E::G>>> {
        let mut q = self;
        q = q.select_only();
        q = q.select_column(col);
        for l in look_ahead {
            if let Some(c) = l.col
                && c.as_str() != col.as_str()
            {
                q = q.select_column(c);
            }
            if let Some(expr) = l.expr.clone() {
                q = q.column_as(expr, l.c);
            }
        }
        let r = q.into_model::<E::G>();
        Ok(r)
    }
    fn gql_select(self, ctx: &Context<'_>) -> Res<Selector<SelectModel<E::G>>> {
        let look_ahead = E::gql_look_ahead(ctx)?;
        self.gql_select_with_look_ahead(&look_ahead, E::col_id())
    }

    fn gql_select_id(self) -> Selector<SelectModel<E::G>> {
        self.select_only().column(E::col_id()).into_model::<E::G>()
    }

    async fn exists_or_404<D>(self, tx: &D) -> Res<()>
    where
        D: ConnectionTrait,
    {
        if !PaginatorTrait::exists(self, tx).await? {
            return Err(MyErr::Db404.into());
        }
        Ok(())
    }
}
