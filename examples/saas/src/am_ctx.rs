use grand_line::prelude::*;

use crate::models::ensure_authenticated;

// ============================================================================
// IntoAmCtx / AmExecCtx - sets audit fields (created/updated/deleted_by_id)
// from the current session, and runs the db operation. This mirrors what
// packages/auth used to provide generically via ctx.auth(); now that auth is
// application code, this small trait just needs our own
// ensure_authenticated() instead. The #[create]/#[update]/#[delete] macros
// call `.exec(ctx)` (not `.exec_without_ctx(tx)`) whenever a resolver is also
// `authz(...)`-guarded, so this needs to exist wherever those macros are used.
// ============================================================================

#[async_trait]
pub trait IntoAmCtx<A> {
    async fn into_am(self, ctx: &Context<'_>) -> Res<A>;
}

#[async_trait]
impl<E, A> IntoAmCtx<A> for AmWrapper<AmCreate, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    async fn into_am(self, ctx: &Context<'_>) -> Res<A> {
        let mut am = self.into_am_without_ctx();
        if E::col_created_by_id().is_some() && am.get_created_by_id().is_not_set() {
            let user_id = ensure_authenticated(ctx).await?.user_id;
            am = am.set_created_by_id(Some(user_id));
        }
        Ok(am)
    }
}

#[async_trait]
impl<E, A> IntoAmCtx<A> for AmWrapper<AmUpdate, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    async fn into_am(self, ctx: &Context<'_>) -> Res<A> {
        let mut am = self.into_am_without_ctx();
        if E::col_updated_by_id().is_some() && !am.get_updated_by_id().is_set() {
            let user_id = ensure_authenticated(ctx).await?.user_id;
            am = am.set_updated_by_id(Some(user_id));
        }
        Ok(am)
    }
}

#[async_trait]
impl<E, A> IntoAmCtx<A> for AmWrapper<AmSoftDelete, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    async fn into_am(self, ctx: &Context<'_>) -> Res<A> {
        let mut am = self.into_am_without_ctx();
        if E::col_deleted_by_id().is_some() && !am.get_deleted_by_id().is_set() {
            let user_id = ensure_authenticated(ctx).await?.user_id;
            am = am.set_deleted_by_id(Some(user_id));
        }
        Ok(am)
    }
}

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
            let by_id = ensure_authenticated(ctx).await.ok().map(|ls| ls.user_id);
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
            let by_id = ensure_authenticated(ctx).await.ok().map(|ls| ls.user_id);
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
            let by_id = ensure_authenticated(ctx).await.ok().map(|ls| ls.user_id);
            History::add(tx, HistoryOperation::Delete, &r, by_id).await?;
        }
        Ok(r)
    }
}
