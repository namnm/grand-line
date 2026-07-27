use super::prelude::*;

/// Abstract extra Select async methods implementation.
/// Make it simpler to also implement for Selector<SelectModel<G>>.
#[async_trait]
pub trait SelectorX<G>
where
    G: FromQueryResult + Send + Sync,
{
    /// Helper to find one and return error if not.
    async fn one_or_404<D>(self, tx: &D) -> Res<G>
    where
        D: ConnectionTrait;
}

/// Automatically implement for Select<E>.
#[async_trait]
impl<E> SelectorX<E::M> for Select<E>
where
    E: EntityX,
{
    async fn one_or_404<D>(self, tx: &D) -> Res<E::M>
    where
        D: ConnectionTrait,
    {
        let v = self.one(tx).await?.ok_or(MyErr::Db404)?;
        Ok(v)
    }
}

/// Automatically implement for Selector<SelectModel<G>>.
#[async_trait]
impl<G> SelectorX<G> for Selector<SelectModel<G>>
where
    G: FromQueryResult + Send + Sync,
{
    async fn one_or_404<D>(self, tx: &D) -> Res<G>
    where
        D: ConnectionTrait,
    {
        let v = self.one(tx).await?.ok_or(MyErr::Db404)?;
        Ok(v)
    }
}
