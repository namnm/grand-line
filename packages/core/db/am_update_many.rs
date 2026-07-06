use super::prelude::*;

// ============================================================================
// AmUpdateMany, Vec<AmWrapper<AmUpdate, E, A>> wrapper backing
// am_update_many!. sea_orm has no single-statement bulk update for rows with
// distinct values, so this just runs one UPDATE per row, reusing the
// single-item exec.

pub struct AmUpdateMany<E, A> {
    items: Vec<AmWrapper<AmUpdate, E, A>>,
}

impl<E, A> AmUpdateMany<E, A> {
    pub const fn new(items: Vec<AmWrapper<AmUpdate, E, A>>) -> Self {
        Self {
            items,
        }
    }

    pub fn into_parts(self) -> Vec<AmWrapper<AmUpdate, E, A>> {
        self.items
    }
}

#[async_trait]
impl<E, A> AmExecWithoutCtx for AmUpdateMany<E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    type Model = Vec<E::M>;

    async fn exec_without_ctx<D>(self, tx: &D) -> Res<Self::Model>
    where
        D: ConnectionTrait,
    {
        exec_each_without_ctx(self.into_parts(), tx).await
    }
}
