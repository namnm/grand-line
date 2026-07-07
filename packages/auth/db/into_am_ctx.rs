use crate::prelude::*;

// ============================================================================
// IntoAmCtx - sets audit fields (created/updated/deleted_by_id) from ctx

/// Converts an active-model wrapper into the plain active model, filling in the
/// created/updated/deleted_by_id audit fields from the context's authenticated user
/// when the entity supports them and the field was not already set explicitly.
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
            let user_id = ctx.auth().await?;
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
        // !is_set() intentionally: every update should attribute updated_by_id
        // to the current actor, including when the ActiveModel came from
        // ..model.into_active_model() and the field is Unchanged rather than NotSet.
        if E::col_updated_by_id().is_some() && !am.get_updated_by_id().is_set() {
            let user_id = ctx.auth().await?;
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
        // !is_set() intentionally, see the above impl for why.
        if E::col_deleted_by_id().is_some() && !am.get_deleted_by_id().is_set() {
            let user_id = ctx.auth().await?;
            am = am.set_deleted_by_id(Some(user_id));
        }
        Ok(am)
    }
}

// ============================================================================
// IntoAmCtx<Vec<A>> for Vec<AmWrapper<T, E, A>>, backing
// AmCreateMany / AmUpdateMany / AmSoftDeleteMany's exec (see am_exec_ctx.rs).
// Same trait as the single-item impls above, just targeting Vec<A> instead of
// A. The ctx audit id is resolved once for the whole batch, not once per row.

#[async_trait]
impl<E, A> IntoAmCtx<Vec<A>> for Vec<AmWrapper<AmCreate, E, A>>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    async fn into_am(self, ctx: &Context<'_>) -> Res<Vec<A>> {
        let mut ams: Vec<A> = self.into_iter().map(|w| w.into_am_without_ctx()).collect();

        if E::col_created_by_id().is_some() && ams.iter().any(|am| am.get_created_by_id().is_not_set()) {
            let user_id = ctx.auth().await?;
            ams = ams
                .into_iter()
                .map(|mut am| {
                    if am.get_created_by_id().is_not_set() {
                        am = am.set_created_by_id(Some(user_id.clone()));
                    }
                    am
                })
                .collect();
        }

        Ok(ams)
    }
}

#[async_trait]
impl<E, A> IntoAmCtx<Vec<A>> for Vec<AmWrapper<AmUpdate, E, A>>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    async fn into_am(self, ctx: &Context<'_>) -> Res<Vec<A>> {
        let mut ams: Vec<A> = self.into_iter().map(|w| w.into_am_without_ctx()).collect();

        // !is_set() intentionally, see the above impl for why.
        if E::col_updated_by_id().is_some() && ams.iter().any(|am| !am.get_updated_by_id().is_set()) {
            let user_id = ctx.auth().await?;
            ams = ams
                .into_iter()
                .map(|mut am| {
                    if !am.get_updated_by_id().is_set() {
                        am = am.set_updated_by_id(Some(user_id.clone()));
                    }
                    am
                })
                .collect();
        }

        Ok(ams)
    }
}

#[async_trait]
impl<E, A> IntoAmCtx<Vec<A>> for Vec<AmWrapper<AmSoftDelete, E, A>>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    async fn into_am(self, ctx: &Context<'_>) -> Res<Vec<A>> {
        let mut ams: Vec<A> = self.into_iter().map(|w| w.into_am_without_ctx()).collect();

        // !is_set() intentionally, see the above impl for why.
        if E::col_deleted_by_id().is_some() && ams.iter().any(|am| !am.get_deleted_by_id().is_set()) {
            let user_id = ctx.auth().await?;
            ams = ams
                .into_iter()
                .map(|mut am| {
                    if !am.get_deleted_by_id().is_set() {
                        am = am.set_deleted_by_id(Some(user_id.clone()));
                    }
                    am
                })
                .collect();
        }

        Ok(ams)
    }
}
