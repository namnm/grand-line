# Resolver bodies, context, and transactions

## Resolver bodies

Resolver bodies are blocks, not functions - `return` only works with errors. `ctx: &Context<'_>` and `db: &ConnX<'_>` are always injected.

```rs
#[query]
fn my_query() -> String {
    if missing {
        return Err(MyErr::NotFound.into()); // ok - return only works for errors
    }
    "ok".to_string() // tail expression is the actual return value
}
```

For bodies that must return `Search`/`Count`/`Detail` (`#[search]`, `#[count]`, `#[detail]`, `#[many_resolver]`, `#[count_resolver]`), if the last statement isn't a tail expression the macro appends `Default::default()` automatically - a body with no extra condition can be left empty:

```rs
#[detail(Todo)]
fn resolver() {
    println!("todoDetail id={id}");
    // no tail expression needed - Detail::default() is appended automatically
}
```

**Caveat:** this check is syntactic (does the last statement lack a trailing semicolon), not a type check. A stray trailing semicolon after what was meant to be the tail expression silently discards it and appends `Default::default()` instead of failing to compile - double-check you haven't left one on your last line in these bodies. This only affects the _outermost_ statement position: an `if`/`match` used correctly as the tail is unaffected, and if a branch of that `if`/`match` ends in a semicolon instead of an expression, the compiler still catches the resulting type mismatch normally (the macro doesn't reach into nested blocks to paper over it).

## Context

`ctx` is injected into every resolver. Core methods, always available:

```rs
ctx.db().await?                       // ConnX - transaction or pooled connection
ctx.tx().await?                       // ConnX - forces the transaction open
ctx.db_pool().await?                  // &DatabaseConnection - the pool itself
ctx.cache(|| async { ... }).await?    // Arc<T> - per-request memoize by type
```

Auth (`auth` feature) and authz (`authz` feature) add their own methods to `ctx` - see [Authentication](authentication.md) and [Authorization](authorization.md).

## Guards (`check`)

`check` runs one or more `ctx` methods before the resolver body and aborts the resolver if any of them returns an error. The macro emits nothing but the call, so the logic stays in a plain trait you implement on `Context` and can unit test on its own:

```rs
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Plan {
    Free,
    Pro,
}

#[async_trait]
pub trait BillingCheck {
    /// Passes when the request resolves to any subscribed workspace.
    async fn subscribed(&self) -> Res<()>;
    /// Passes only when the resolved workspace is on the required plan.
    async fn plan(&self, required: Plan) -> Res<()>;
}

#[async_trait]
impl BillingCheck for Context<'_> {
    async fn subscribed(&self) -> Res<()> {
        // .. your own lookup, through ctx.db()
        Ok(())
    }
    async fn plan(&self, required: Plan) -> Res<()> {
        // ..
        Ok(())
    }
}
```

Put the trait in your app's `prelude` and name it in the attribute:

| Attribute                            | Generated                     |
| ------------------------------------ | ----------------------------- |
| `check = subscribed`                 | `ctx.subscribed().await?;`    |
| `check = plan(Plan::Pro)`            | `ctx.plan(Plan::Pro).await?;` |
| `check(subscribed, plan(Plan::Pro))` | both, in the order written    |

```rs
#[query(check = subscribed)]
fn my_query() -> bool {
}

#[search(Task, check = plan(Plan::Pro))]
fn resolver() {
}
```

Works on `#[query]`, `#[mutation]`, and every crud macro. Notes:

- Guards run before the body and after `db` is available, so a guard can query through `ctx.db()`. `ctx = false` with a `check` is a compile error.
- The value a guard returns is discarded, only its `?` matters, so any `Res<T>` signature works.
- Only a bare name or a call is accepted - `check = a::b` and `check = x.y()` are rejected at macro expansion.
- A guard is the only place a resolver states who may call it. The `created_by_id`/`updated_by_id`/`deleted_by_id` audit fields are independent of it - with the `auth` feature on they are filled from `ctx.auth()` whenever the request happens to be authenticated, and left unset otherwise.
- Two authz guards on one resolver are both evaluated, each on its own realm/org/user requirements, so the resolver runs only if all of them pass. What follows the guards - `ctx.authz_role()` and the row policy it carries - comes from the **first** one listed, so put the guard whose row policy should apply first.

## Connections and transactions

`GrandLineExtension` decides what a request needs from its operation type, before any resolver runs.

| Operation      | `db` is                 | Cost                                                     |
| -------------- | ----------------------- | -------------------------------------------------------- |
| `mutation`     | the request transaction | one `BEGIN`, commits on success, rolls back on any error |
| `query`        | a pooled connection     | none, no transaction is ever opened                      |
| `subscription` | a pooled connection     | none, see [Subscriptions](subscriptions.md)              |

```rs
GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
    .extension(GrandLineExtension)
    .data(Arc::new(db.clone()))
    .finish()
```

The injected `db` is a `ConnX`, not a raw pool handle. It implements sea-orm's `ConnectionTrait`, so it goes wherever a connection goes: `.one(db)`, `.all(db)`, `.exec(db)`.

Three names, easy to mix up, so worth stating once:

|                             | Is                                     | Use it for                           |
| --------------------------- | -------------------------------------- | ------------------------------------ |
| `db` (injected), `ctx.db()` | `ConnX`, the request's own connection  | everything                           |
| `ctx.tx()`                  | `ConnX`, forced onto the transaction   | a query that turns out to write      |
| `ctx.db_pool()`             | `&DatabaseConnection`, the pool itself | a write that must survive a rollback |

It is a borrow tied to the request, never an owning handle, and that is what makes the commit safe. Committing consumes the transaction, so anything still holding one would block it; with a borrow the compiler refuses to let a connection outlive the request in the first place, rather than leaving it as a rule to remember.

A query that turns out to write forces the transaction open with `ctx.tx()`. Every later `ctx.db()` in that request returns the transaction too, so a read never sees around a write that preceded it:

```rs
#[query]
fn my_query() -> bool {
    ctx.tx().await?; // from here on this request is transactional
    true
}
```

`ctx.db_pool()` steps outside the request entirely. The otp attempt counter is the framework's own use of it, so a failed resolve still counts against the limit after the request rolls back, see [Authentication](authentication.md).

Two things follow from that, and they are the whole point of `db_pool()` and also its whole risk:

- **Anything written through it is outside the request's rollback.** That is what makes it right for an attempt counter and wrong for everything else. A partial write it leaves behind is not undone when the resolver errors.
- **On sqlite it can deadlock against your own request.** Writing through `ctx.db_pool()` while the request transaction already holds the write lock returns `SQLITE_BUSY`. The otp flow is safe because its writes happen before the transaction takes a lock, which is an accident of ordering, not a guarantee. Postgres does not have this problem, it blocks per row rather than per database.

## Work that outlives the request

The transaction is held for the whole resolver body, so a mutation that kicks off something slow would hold a connection open for the duration. `ctx.detach()` queues that work instead:

```rs
#[mutation]
fn transcode_video(id: String) -> bool {
    ctx.detach(move |db| async move {
        // db is a pooled connection, the request transaction is already gone
        transcode(&id).await?;
        am_update!(Video {
            id: id,
            status: VideoStatus::Ready,
        })
        .exec_without_ctx(db.as_ref())
        .await?;
        Ok(())
    })
    .await?;
    true
}
```

The job is spawned only after the request commits. A rollback drops it: there is no background work to do for a request that did not land. It is handed a pooled connection rather than the request transaction, so it cannot race the commit or write through a transaction the request owns.

Do not reach for a bare `tokio::spawn` here. A task spawned from a resolver runs while the request transaction is still open, and on a fast path it can try to `UPDATE` a row the request has not committed yet: postgres blocks until the commit, sqlite returns `database is locked`, and if the request then rolls back the task's write stands on its own.

## What a rollback does to the response

When a mutation errors, the transaction rolls back and the response's `data` is set to `null`. Everything the resolvers produced before the error describes rows that no longer exist, and a client reading only `data` would take them as written.

A query is left alone. It opens no transaction, so an error in one field undid nothing and GraphQL's partial success still applies: the failed field is `null`, its siblings keep their values.

**Known limitation:** inside a mutation every resolver shares the one transaction, i.e. one underlying DB connection. Sibling GraphQL fields, relation resolvers included, may be scheduled concurrently as Rust futures, but their statements still serialize on that connection. Queries no longer have this problem, they read from the pool.
