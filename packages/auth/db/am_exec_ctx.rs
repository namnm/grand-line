use crate::prelude::*;

// ============================================================================
// AmExecCtx - resolves ctx, builds active model, runs db operation
// ============================================================================

/// Resolves the context's audit id, builds the active model, and runs the db operation,
/// recording a History entry when the target entity has history enabled.
#[async_trait]
pub trait AmExecCtx
where
    Self: Sized,
{
    type Model: Send + Sync;
    async fn exec(self, ctx: &Context<'_>) -> Res<Self::Model>;
}

#[async_trait]
impl<E, A> AmExecCtx for AmWrapper<AmCreate, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    type Model = E::M;

    async fn exec(self, ctx: &Context<'_>) -> Res<Self::Model> {
        let am = self.into_am(ctx).await?;
        let tx = &*ctx.tx().await?;
        let r = am.insert(tx).await?;
        if E::has_history() {
            let by_id = ctx.auth().await.ok();
            History::add(tx, HistoryOperation::Create, &r, by_id).await?;
        }
        Ok(r)
    }
}

#[async_trait]
impl<E, A> AmExecCtx for AmWrapper<AmUpdate, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    type Model = E::M;

    async fn exec(self, ctx: &Context<'_>) -> Res<Self::Model> {
        let am = self.into_am(ctx).await?;
        am.ensure_id_set()?;
        let tx = &*ctx.tx().await?;
        let r = am.update(tx).await?;
        if E::has_history() {
            let by_id = ctx.auth().await.ok();
            History::add(tx, HistoryOperation::Update, &r, by_id).await?;
        }
        Ok(r)
    }
}

#[async_trait]
impl<E, A> AmExecCtx for AmWrapper<AmSoftDelete, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    type Model = E::M;

    async fn exec(self, ctx: &Context<'_>) -> Res<Self::Model> {
        let am = self.into_am(ctx).await?;
        am.ensure_id_set()?;
        let tx = &*ctx.tx().await?;
        let r = am.update(tx).await?;
        if E::has_history() {
            let by_id = ctx.auth().await.ok();
            History::add(tx, HistoryOperation::Delete, &r, by_id).await?;
        }
        Ok(r)
    }
}

// ============================================================================
// AmExecCtx for AmCreateMany / AmUpdateMany / AmSoftDeleteMany, backing
// am_create_many! / am_update_many! / am_soft_delete_many!. Same trait and
// method name as the single-item impls above, just a different receiver type.
// The ctx-resolved audit id and history by_id are looked up once per batch
// instead of once per row.

#[async_trait]
impl<E, A> AmExecCtx for AmCreateMany<E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    type Model = Vec<E::M>;

    async fn exec(self, ctx: &Context<'_>) -> Res<Self::Model> {
        let (items, returning) = self.into_parts();
        let ams = items.into_am(ctx).await?;

        let tx = &*ctx.tx().await?;
        let models = insert_many_with_returning(ams, returning, tx).await?;

        if E::has_history() {
            let by_id = ctx.auth().await.ok();
            History::add_many(tx, HistoryOperation::Create, &models, by_id).await?;
        }
        Ok(models)
    }
}

#[async_trait]
impl<E, A> AmExecCtx for AmUpdateMany<E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    type Model = Vec<E::M>;

    async fn exec(self, ctx: &Context<'_>) -> Res<Self::Model> {
        let by_id = ctx.auth().await.ok();
        let ams = self.into_parts().into_am(ctx).await?;
        let tx = &*ctx.tx().await?;

        let mut models = vec![];
        for am in ams {
            am.ensure_id_set()?;
            models.push(am.update(tx).await?);
        }
        if E::has_history() {
            History::add_many(tx, HistoryOperation::Update, &models, by_id).await?;
        }
        Ok(models)
    }
}

#[async_trait]
impl<E, A> AmExecCtx for AmSoftDeleteMany<E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    type Model = Vec<E::M>;

    async fn exec(self, ctx: &Context<'_>) -> Res<Self::Model> {
        let by_id = ctx.auth().await.ok();
        let ams = self.into_parts().into_am(ctx).await?;
        let tx = &*ctx.tx().await?;

        let mut models = vec![];
        for am in ams {
            am.ensure_id_set()?;
            models.push(am.update(tx).await?);
        }
        if E::has_history() {
            History::add_many(tx, HistoryOperation::Delete, &models, by_id).await?;
        }
        Ok(models)
    }
}
