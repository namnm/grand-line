use super::prelude::*;

// ============================================================================
// exec_without_ctx
// ============================================================================

/// Execute a single active-model wrapper without a Ctx, persisting the row
/// (insert, update, or soft delete depending on the operation kind) and
/// returning the resulting model.
#[async_trait]
pub trait AmExecWithoutCtx {
    type Model: Send + Sync;

    /// Persist this wrapper's active model via tx and return the saved model.
    async fn exec_without_ctx<D>(self, tx: &D) -> Res<Self::Model>
    where
        D: ConnectionTrait;
}

#[async_trait]
impl<E, A> AmExecWithoutCtx for AmWrapper<AmCreate, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    type Model = E::M;

    async fn exec_without_ctx<D>(self, tx: &D) -> Res<Self::Model>
    where
        D: ConnectionTrait,
    {
        let r = self.into_am_without_ctx().insert(tx).await?;
        if E::has_history() {
            History::add(tx, HistoryOperation::Create, &r, None).await?;
        }
        Ok(r)
    }
}

#[async_trait]
impl<E, A> AmExecWithoutCtx for AmWrapper<AmUpdate, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    type Model = E::M;

    async fn exec_without_ctx<D>(self, tx: &D) -> Res<Self::Model>
    where
        D: ConnectionTrait,
    {
        let am = self.into_am_without_ctx();
        am.ensure_id_set()?;
        let r = am.update(tx).await?;
        if E::has_history() {
            History::add(tx, HistoryOperation::Update, &r, None).await?;
        }
        Ok(r)
    }
}

#[async_trait]
impl<E, A> AmExecWithoutCtx for AmWrapper<AmSoftDelete, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    type Model = E::M;

    async fn exec_without_ctx<D>(self, tx: &D) -> Res<Self::Model>
    where
        D: ConnectionTrait,
    {
        E::ensure_col_deleted_at()?;
        let am = self.into_am_without_ctx();
        am.ensure_id_set()?;
        let r = am.update(tx).await?;
        if E::has_history() {
            History::add(tx, HistoryOperation::Delete, &r, None).await?;
        }
        Ok(r)
    }
}

// ---------------------------------------------------------------------------
// Shared helper for bulk multi-item exec
// ---------------------------------------------------------------------------

/// Run exec_without_ctx on each item in order, collecting the results.
/// Shared by AmUpdateMany / AmSoftDeleteMany, sea_orm has no single-statement
/// bulk update for rows with distinct values, so these just run one op per row.
pub async fn exec_each_without_ctx<W, D>(items: Vec<W>, tx: &D) -> Res<Vec<W::Model>>
where
    W: AmExecWithoutCtx,
    D: ConnectionTrait,
{
    let mut models = vec![];
    for w in items {
        models.push(w.exec_without_ctx(tx).await?);
    }
    Ok(models)
}
