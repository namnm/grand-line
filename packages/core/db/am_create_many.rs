use super::prelude::*;

// ============================================================================
// AmCreateMany, Vec<AmWrapper<AmCreate, E, A>> plus an opt-in
// returning() flag, backing am_create_many!.
//
// Default (returning=false): reconstruct the models in memory (try_into_model)
// with no round trip after the bulk INSERT, ids/defaults are already known
// client-side. Opt into returning() when the row needs to reflect DB-side
// defaults/triggers (custom db) that the client can't see.

pub struct AmCreateMany<E, A> {
    items: Vec<AmWrapper<AmCreate, E, A>>,
    returning: bool,
}

impl<E, A> AmCreateMany<E, A> {
    pub const fn new(items: Vec<AmWrapper<AmCreate, E, A>>) -> Self {
        Self {
            items,
            returning: false,
        }
    }

    pub const fn returning(mut self) -> Self {
        self.returning = true;
        self
    }

    pub fn into_parts(self) -> (Vec<AmWrapper<AmCreate, E, A>>, bool) {
        (self.items, self.returning)
    }
}

/// A fresh row has no updated/deleted state yet, settle these to None instead of
/// leaving them NotSet. Same DB row either way (all nullable columns, omitted from
/// the INSERT vs explicit NULL), but it lets ActiveModelTrait::try_into_model
/// succeed for the fast in-memory (non-returning) path below. Only applied to the
/// in-memory copy used to build the returned models, not to what's actually sent
/// to insert_many, so it can't affect the id/audit fields into_am(ctx) resolved.
fn settle_nullable_defaults<E, A>(mut am: A) -> A
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    if am.get_updated_at().is_not_set() {
        am = am.set_updated_at(None);
    }
    if am.get_deleted_at().is_not_set() {
        am = am.set_deleted_at(None);
    }
    if am.get_created_by_id().is_not_set() {
        am = am.set_created_by_id(None);
    }
    if am.get_updated_by_id().is_not_set() {
        am = am.set_updated_by_id(None);
    }
    if am.get_deleted_by_id().is_not_set() {
        am = am.set_deleted_by_id(None);
    }
    am
}

/// Bulk insert already-built active models, honoring the returning() opt-in:
/// - returning = false, reconstruct the models in memory (try_into_model), no extra
///   round trip, but this can't reflect DB-side defaults/triggers.
/// - returning = true, use RETURNING on postgres, or an extra SELECT by id on other
///   backends (needed when the caller has custom db defaults/triggers).
pub async fn insert_many_with_returning<E, A, D>(ams: Vec<A>, returning: bool, tx: &D) -> Res<Vec<E::M>>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
    D: ConnectionTrait,
{
    if ams.is_empty() {
        return Ok(vec![]);
    }

    if !returning {
        let models = ams
            .iter()
            .cloned()
            .map(|am| settle_nullable_defaults(am).try_into_model())
            .collect::<Result<Vec<_>, _>>()?;
        E::insert_many(ams).exec_without_returning(tx).await?;
        return Ok(models);
    }

    if cfg!(feature = "postgres") {
        let models = E::insert_many(ams).exec_with_returning_many(tx).await?;
        return Ok(models);
    }

    // ids are already generated client-side by set_defaults_on_create, collect them
    // before insert_many consumes the active models.
    let ids = ams
        .iter()
        .map(|am| match am.get_id() {
            Set(id) => Ok(id),
            _ => Err(MyErr::IdNotSet),
        })
        .collect::<Result<Vec<_>, _>>()?;
    E::insert_many(ams).exec_without_returning(tx).await?;

    let mut by_id: HashMap<String, E::M> = E::find()
        .filter(E::col_id().is_in(ids.clone()))
        .all(tx)
        .await?
        .into_iter()
        .map(|m| (m.get_id(), m))
        .collect();
    let models = ids
        .into_iter()
        .map(|id| by_id.remove(&id).ok_or(MyErr::Db404))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(models)
}

#[async_trait]
impl<E, A> AmExecWithoutCtx for AmCreateMany<E, A>
where
    E: EntityX<A = A>,
    A: ActiveModelX<E>,
{
    type Model = Vec<E::M>;

    async fn exec_without_ctx<D>(self, tx: &D) -> Res<Self::Model>
    where
        D: ConnectionTrait,
    {
        let (items, returning) = self.into_parts();
        let ams = items.into_iter().map(|w| w.into_am_without_ctx()).collect::<Vec<A>>();
        insert_many_with_returning(ams, returning, tx).await
    }
}
