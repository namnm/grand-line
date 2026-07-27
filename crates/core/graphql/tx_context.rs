use super::prelude::*;

/// Access to the per-request database transaction stored in GrandLineData.
#[async_trait]
pub trait TxContext<'a>
where
    Self: GrandLineDataContext<'a>,
{
    /// Shortcut to get tx from grand line data.
    async fn tx(&self) -> Res<Arc<DatabaseTransaction>> {
        self.grand_line()?.tx().await
    }
}

#[async_trait]
impl<'a> TxContext<'a> for Context<'a> {
}
