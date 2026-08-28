# Subscriptions

The `subscription` feature turns every crud mutation into an event source and gives each model a root subscription field, so a client can watch a table without any polling.

## Setup

Nothing to register for a single instance - the in memory broker is the default:

```rs
#[model]
pub struct Todo {
    pub content: String,
    #[default(false)]
    pub done: bool,
}

#[create(Todo)]
fn resolver() {
    am_create!(Todo {
        content: data.content,
    })
}

#[subscribe(Todo)]
fn resolver() {
}
```

```graphql
subscription {
  todoChanged {
    created { id content done }
    updated { id content }
    deleted { id }
  }
}
```

`#[create]` / `#[update]` / `#[delete]` publish on their own, so those three lines are the whole cost. The [schema collector](schema-collector.md) picks up `*Subscription` types the same way it does queries and mutations, and generates `pub struct Subscription(..)`:

```rs
pub type AppSchema = GraphQLSchema<Query, Mutation, Subscription>;

GraphQLSchema::build(Query::default(), Mutation::default(), Subscription::default())
    .extension(GrandLineExtension)
    .data(Arc::new(db.clone()))
    .finish()
```

## The event

`#[subscribe(Todo)]` generates one field plus its payload type, named after the field so two subscriptions on one model do not collide. The payload has one slot per operation, and exactly one of them is set:

| Field     | Type   | Set when          |
| --------- | ------ | ----------------- |
| `created` | `Todo` | a row was created |
| `updated` | `Todo` | a row was updated |
| `deleted` | `Todo` | a row was deleted |

One slot per operation rather than an operation enum plus a shared node, so the client picks its own selection per case: the whole row on a create, the changed columns on an update, just the id on a delete.

**Selecting a slot is how you subscribe to that operation.** An event whose slot the client did not select is dropped before any query runs, so `todoChanged { created { id } }` never wakes up for an update.

Each slot is reloaded per event on a pooled connection, never the request transaction, and only selects the columns that slot actually asked for.

A soft deleted row is still reloaded for `deleted`, so every field the client selects resolves normally. A permanently deleted row is different: there is nothing left to select from, and nothing to check the subscription's filters against either. It is dropped by default, so a filtered subscription can never leak the id of a row it was not allowed to see. Opt in per subscription when the client needs to know an id is gone:

```rs
#[subscribe(Todo, allow_permanent_delete)]
fn resolver() {
}
```

The name says what it allows: those events carry only the id, so `deleted { id }` resolves and any other field errors as an unresolved value, and **the filters are not applied to them**. Only turn it on where every id is fair game for every subscriber. The framework soft deletes by default, so this is limited to `permanent: true` hard deletes.

## Filtering

Two layers, both optional. The client passes `filter`, the same input type `todoSearch` takes:

```graphql
subscription {
  todoChanged(filter: { done: false }) {
    created { id }
    updated { id }
  }
}
```

The resolver body contributes a server side `Detail`, the same shape `#[detail]` uses, ANDed with whatever the client sent:

```rs
#[subscribe(Todo, check = authz_org)]
fn todo_pending_changed() {
    filter!(Todo {
        done: false,
    })
    .into()
}
```

An event whose row no longer matches either layer is skipped rather than reported, so a filter is a real boundary and not just a hint. [Guards](resolvers.md#guards-check) work exactly as they do on a query, and run once when the client subscribes.

## Publishing from your own mutations

A hand written `#[mutation]` publishes what it wants:

```rs
#[mutation]
fn todo_delete_done() -> Vec<TodoGql> {
    let f = filter!(Todo {
        done: true,
    });
    let r = f.clone().into_select().all(db).await?;
    Todo::soft_delete_many()?.filter(f).exec(db).await?;
    for t in &r {
        ctx.subscription_queue::<Todo>(SubscriptionOperation::Delete, &t.id)
            .await?;
    }
    r.iter().map(|t| TodoGql::from_id(&t.id)).collect()
}
```

`subscription_queue` only queues. The extension publishes the queue after the request transaction commits, so **a rolled back request publishes nothing** - a subscriber never hears about a change that did not land.

## Publishing with no request at all

A cron job, a migration, or a separate worker process has no `ctx`. Build the same `SubscriptionConfig` the schema uses and publish through it directly:

```rs
let subscription = SubscriptionConfig::new(SubscriptionBroker::Redis(SubscriptionRedis::url(
    "redis://localhost:6379",
)));

// on the schema
GraphQLSchema::build(..).data(subscription.clone())

// and from anywhere else, including another binary
subscription
    .publish::<Todo>(SubscriptionOperation::Update, &id)
    .await?;
```

`publish` sends immediately, with no queue and no commit to wait for, so the caller owns the ordering: write first, publish after.

Note the `.clone()`: with `SubscriptionBroker::InMemory` the channel belongs to that config, not to the process, so two schemas built from two configs never hear each other. Publishing has to go through the very config the schema was built with, cloned rather than rebuilt. A worker in its own binary cannot share an object at all, so it needs the redis broker to be heard.

Turn a crud resolver's own publishing off per resolver:

```rs
#[create(Todo, publish = false)]
fn todo_import() {
}
```

## Brokers

`SubscriptionBroker` is the only thing to choose between one instance and many:

```rs
// default, no registration needed
SubscriptionConfig::default()

// or, redis pub/sub, credentials in the url
.data(SubscriptionConfig::new(SubscriptionBroker::Redis(SubscriptionRedis::url(
    "redis://user:password@localhost:6379/0",
))))
```

| Variant                          | Reaches                                 | Feature              |
| -------------------------------- | --------------------------------------- | -------------------- |
| `SubscriptionBroker::InMemory`   | subscribers of this process only        | `subscription`       |
| `SubscriptionBroker::Redis(..)`  | every instance on the same redis server | `subscription_redis` |
| `SubscriptionBroker::Custom(..)` | whatever you implement                  | `subscription`       |

`InMemory` is correct for a single process and silently partial once there are two, so switch to `Redis` before scaling out horizontally. `SubscriptionRedis` also takes a `channel_prefix` (default `grand_line:sub:`) so several apps can share one redis server without hearing each other's events.

For a transport the framework does not ship, implement `SubscriptionBrokerImpl` and wrap it:

```rs
#[async_trait]
impl SubscriptionBrokerImpl for MyBroker {
    async fn publish(&self, e: SubscriptionEvent) -> Res<()> { /* ... */ }
    fn subscribe(&self, entity: &'static str) -> BoxStream<'static, SubscriptionEvent> { /* ... */ }
}

.data(SubscriptionConfig::new(SubscriptionBroker::Custom(Arc::new(MyBroker))))
```

## Guards, transactions, and staleness

A subscription runs its guards and its body exactly once, when the client subscribes, before any event. An error there yields one response carrying the error and closes the subscription - guards never fire per event, and a subscriber is never handed a mid stream authorization error.

What that costs, and what to do about it:

**A subscription never opens a transaction.** Its operation type says read, so `ctx.db()` hands out a pooled connection to the guards, to every per event reload, and to any relation the payload selects through the dataloader. Nothing is pinned for the life of the stream. `#[subscribe]` still calls `ctx.tx_finish()` after its guards, which only matters when a guard forced one open with `ctx.tx()`, and `GrandLineExtension::subscribe` releases anything left when the stream ends or the client disconnects.

**Permissions are frozen for the life of the subscription.** The per request cache that holds the matched role, its col/row policy, and the resolved session lives as long as the subscription does. An admin revoking a role, editing a policy, or deleting a session has no effect on a subscription that is already open: it keeps receiving under the permissions it was granted at subscribe time.

The row filter still applies on every event, so a subscriber never sees a row outside the boundary the body computed - but that boundary itself was computed once. Until the framework offers a re-check, bound the exposure at the edge:

- Close subscriptions on the gateway or client after a fixed lifetime and let the client resubscribe, which re-runs every guard from scratch.
- Where a revocation must take effect immediately, keep the sensitive fields out of the subscription payload and have the client refetch them through a query, which is guarded per request.

The same applies to anything put in `ctx.cache()` from inside a subscription: it is computed once and never refreshed.

## No replay

A subscriber only receives what is published after it is listening, and the broker keeps no history. Two consequences worth designing around:

- The stream is established when the client's subscribe reaches the server and the resolver runs, not when the client sends it. Anything that changes in that window is missed.
- A dropped connection loses every event until the client resubscribes.

Both are normal for pub/sub and the fix is the same either way: after subscribing, fetch the current state with a query, and treat the subscription as an invalidation signal rather than the only source of truth.

## Known limits

- A permanently deleted row is dropped unless the subscription opts in with `allow_permanent_delete`, and those events bypass the filters.
- The payload carries the row as it stands when the event is delivered, not a snapshot of when it was published. Two quick writes can deliver the same final state twice.
- The in memory broker buffers 1024 events; a subscriber that falls further behind silently misses the ones in between rather than closing.
- Guards and the per request cache are evaluated once at subscribe time, see above.
