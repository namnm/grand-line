use super::prelude::*;

/// Abstract extra Select async methods implementation.
/// Make it simpler to also implement for Selector<`SelectModel`<G>>.
#[async_trait]
pub trait SelectorX<G>
where
    G: FromQueryResult + Send + Sync,
{
    /// Helper to find one and return error if not.
    async fn one_or_404<D>(self, db: &D) -> Res<G>
    where
        D: ConnectionTrait;
}

/// Automatically implement for Selector<`SelectModel`<G>>.
#[async_trait]
impl<G> SelectorX<G> for Selector<SelectModel<G>>
where
    G: FromQueryResult + Send + Sync,
{
    async fn one_or_404<D>(self, db: &D) -> Res<G>
    where
        D: ConnectionTrait,
    {
        let v = self.one(db).await?.ok_or(MyErr::Db404)?;
        Ok(v)
    }
}
