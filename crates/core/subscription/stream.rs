use super::prelude::*;
use async_graphql::futures_util::stream::{Stream, StreamExt as _};

/// Gql fields a generated event type nests the changed row under, one per
/// operation. Each carries its own selection set, so a client can ask for the
/// whole row on a create and just the id on a delete.
pub const SUBSCRIPTION_CREATED: &str = "created";
pub const SUBSCRIPTION_UPDATED: &str = "updated";
pub const SUBSCRIPTION_DELETED: &str = "deleted";

/// One resolved row change, the #[subscribe] macro turns this into the model's
/// own event type.
pub struct SubscriptionItem<E>
where
    E: EntityX,
{
    operation: SubscriptionOperation,
    node: E::G,
}

impl<E> SubscriptionItem<E>
where
    E: EntityX,
{
    /// Splits the item into the created, updated and deleted slots of a generated
    /// event type, exactly one of which is set.
    pub fn split(self) -> (Option<E::G>, Option<E::G>, Option<E::G>) {
        match self.operation {
            SubscriptionOperation::Create => (Some(self.node), None, None),
            SubscriptionOperation::Update => (None, Some(self.node), None),
            SubscriptionOperation::Delete => (None, None, Some(self.node)),
        }
    }
}

/// What the client asked for, per operation, read once from the subscription's
/// own selection set. None means the operation's field was not selected at all,
/// which is how a client says it does not care about that kind of change.
struct Selected<E>
where
    E: EntityX,
{
    created: Option<Vec<LookaheadX<E>>>,
    updated: Option<Vec<LookaheadX<E>>>,
    deleted: Option<Vec<LookaheadX<E>>>,
}

impl<E> Selected<E>
where
    E: EntityX,
{
    fn of(ctx: &Context<'_>) -> Res<Self> {
        Ok(Self {
            created: Self::field(ctx, SUBSCRIPTION_CREATED)?,
            updated: Self::field(ctx, SUBSCRIPTION_UPDATED)?,
            deleted: Self::field(ctx, SUBSCRIPTION_DELETED)?,
        })
    }

    fn field(ctx: &Context<'_>, field: &str) -> Res<Option<Vec<LookaheadX<E>>>> {
        if !subscription_selected(ctx, field) {
            return Ok(None);
        }
        Ok(Some(E::gql_look_ahead_at(ctx, field)?))
    }

    const fn look_ahead(&self, operation: SubscriptionOperation) -> Option<&Vec<LookaheadX<E>>> {
        match operation {
            SubscriptionOperation::Create => self.created.as_ref(),
            SubscriptionOperation::Update => self.updated.as_ref(),
            SubscriptionOperation::Delete => self.deleted.as_ref(),
        }
    }
}

/// Whether the subscription's own selection set names field.
fn subscription_selected(ctx: &Context<'_>, field: &str) -> bool {
    ctx.look_ahead()
        .selection_fields()
        .first()
        .is_some_and(|f| f.selection_set().any(|c| c.name() == field))
}

/// Turns the broker's raw event stream into a subscription stream, reloading each
/// changed row through the caller's own selection and filters.
///
/// The reload runs on a pooled connection, never the request transaction, which is
/// already finished by the time the first event arrives. An event whose operation
/// the client did not select is dropped before any query runs, and so is a row
/// that no longer matches filter or extra.
pub fn subscription_stream<'a, E>(
    ctx: &'a Context<'a>,
    filter: Option<E::F>,
    extra: Detail,
    allow_permanent_delete: bool,
) -> Res<impl Stream<Item = Res<SubscriptionItem<E>>> + 'a>
where
    E: EntityX,
{
    let selected = Arc::new(Selected::<E>::of(ctx)?);
    let cond = extra.add_option(filter).condition;
    let events = ctx.subscription_config().broker().subscribe(E::model_name());

    let r = events.filter_map(move |e| {
        let cond = cond.clone();
        let selected = Arc::clone(&selected);
        async move {
            let look_ahead = selected.look_ahead(e.operation)?;
            match load::<E>(ctx, &e, cond, look_ahead, allow_permanent_delete).await {
                Ok(Some(i)) => Some(Ok(i)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        }
    });
    Ok(r)
}

/// Reloads the changed row, None when it no longer matches or no longer exists.
/// Soft deleted rows are included on purpose: a delete is a change the subscriber
/// asked to hear about, and its data is still what they should see.
async fn load<E>(
    ctx: &Context<'_>,
    e: &SubscriptionEvent,
    cond: Condition,
    look_ahead: &[LookaheadX<E>],
    allow_permanent_delete: bool,
) -> Res<Option<SubscriptionItem<E>>>
where
    E: EntityX,
{
    let db = ctx.db_pool().await?;
    let node = E::find()
        .include_deleted(true)
        .filter_by_id(&e.id)
        .filter(cond)
        .gql_select_with_look_ahead(look_ahead, E::col_id())?
        .one(db)
        .await?;

    if let Some(node) = node {
        let r = SubscriptionItem {
            operation: e.operation,
            node,
        };
        return Ok(Some(r));
    }

    // The row is gone for good, so there is nothing left to check the filters
    // against. Delivering it anyway tells the client an id existed and was removed
    // even when its row would have been filtered out, which is why it takes an
    // explicit opt in. Only the id is set, so only deleted { id } resolves.
    if allow_permanent_delete && e.operation == SubscriptionOperation::Delete {
        let r = SubscriptionItem {
            operation: e.operation,
            node: E::G::from_id(&e.id),
        };
        return Ok(Some(r));
    }

    Ok(None)
}
