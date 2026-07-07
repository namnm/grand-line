use super::prelude::*;

// ============================================================================
// IntoAmWithoutCtx impls for bulk operations (e.g. Entity::insert_many)
// ============================================================================

/// Convert a bulk-operation wrapper into its underlying active model, applying
/// the create, update, or soft-delete specific defaults without needing a Ctx.
pub trait IntoAmWithoutCtx<A> {
    fn into_am_without_ctx(self) -> A;
}

impl<E, A> IntoAmWithoutCtx<A> for AmWrapper<AmCreate, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    fn into_am_without_ctx(self) -> A {
        self.am.set_defaults_on_create()
    }
}

impl<E, A> IntoAmWithoutCtx<A> for AmWrapper<AmUpdate, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    fn into_am_without_ctx(self) -> A {
        self.am.set_defaults_on_update()
    }
}

impl<E, A> IntoAmWithoutCtx<A> for AmWrapper<AmSoftDelete, E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    fn into_am_without_ctx(self) -> A {
        self.am.set_defaults_on_delete()
    }
}
