# SeaORM query helpers

The traits in `crates/core/db` extend sea-orm's `Select`, `Filter`, and `ActiveModel` types with convenience methods available throughout resolvers.

Available on `Select<E>`, `DeleteMany<E>`, and `UpdateMany<E>`:

```rs
Todo::find()
    .filter_by_id(&id)       // WHERE id = ?
    .include_deleted(false); // WHERE deleted_at IS NULL (no-op if the model has no deleted_at)
```

Available on `Select<E>`:

```rs
Todo::find()
    .filter_option(some_cond)  // filter only if Some
    .filter_option(filter)     // apply a TodoFilter
    .chain(order_by)           // apply a Vec<TodoOrderBy>
    .gql_select(ctx)?          // select only columns requested in the GQL look-ahead
    .gql_select_id()           // select only id (for delete response)
    .exists_or_404(tx).await?; // error if no row matches

Todo::find().one_or_404(tx).await?; // one() + error if None
selector.one_or_404(tx).await?;     // same on Selector<SelectModel<G>>
```

Available on `Filter` and `OrderBy` via `IntoSelect`:

```rs
filter.into_select();    // E::find().filter_option(filter)
filter.gql_select(ctx)?; // shortcut for into_select().gql_select(ctx)
filter.gql_select_id();  // shortcut for into_select().gql_select_id()
```
