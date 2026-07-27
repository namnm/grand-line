# Resolver bodies, context, and transactions

## Resolver bodies

Resolver bodies are blocks, not functions - `return` only works with errors. `ctx: &Context<'_>` and `tx: &DatabaseTransaction` are always injected.

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
ctx.tx().await?                       // Arc<DatabaseTransaction>
ctx.cache(|| async { ... }).await?    // Arc<T> - per-request memoize by type
```

Auth (`auth` feature) and authz (`authz` feature) add their own methods to `ctx` - see [Authentication](authentication.md) and [Authorization](authorization.md).

## Transactions

`GrandLineExtension` manages one lazy transaction per request - commits on success, rolls back on any error.

```rs
GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
    .extension(GrandLineExtension)
    .data(Arc::new(db.clone()))
    .finish()
```

**Known limitation:** all resolvers in a request share one `DatabaseTransaction`, i.e. one underlying DB connection. Sibling GraphQL fields (including sibling relation resolvers) may be scheduled concurrently as Rust futures, but their SQL statements still serialize one at a time on that connection - there is no query-level parallelism within a request today. Giving read-only requests their own pooled connections (instead of one shared transaction) would let sibling relations actually run in parallel; this is not implemented yet. Mutations would keep the single-transaction model for write consistency.
