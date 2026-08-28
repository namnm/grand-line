# Active model helpers

```rs
// auto id, created_at, field defaults
am_create!(Todo {
    content: "hello",
})
// auto updated_at
am_update!(Todo {
    id: id.clone(),
    content: "new",
})
// auto updated_at + deleted_at
am_soft_delete!(Todo {
    id: id.clone(),
})

am.exec(ctx).await?;                // insert/update, sets *_by_id from ctx.auth()
am.exec_without_ctx(db).await?;     // insert/update, no by_id fields
am.into_am(ctx).await?;   // unwrap to raw sea-orm ActiveModel, sets *_by_id from ctx.auth()
am.into_am_without_ctx(); // unwrap to raw sea-orm ActiveModel, no by_id fields

Todo::soft_delete_by_id(&id)?.exec(db).await?;
Todo::soft_delete_many()?.filter(condition).exec(db).await?;
am.soft_delete(db).await?; // on an active model instance
```

`exec_without_ctx` skipping `*_by_id` fields is the documented contract, not a shortcut with a downside - it's meant for seeding and batch jobs that have no GraphQL `Context` to read the current user from (see [Design notes](contribution/design-notes.md)).

`am.exec(ctx)` requires the `auth` feature (it reads `ctx.auth()` to fill `*_by_id`, see [Authentication](authentication.md)) - without it, use `exec_without_ctx` everywhere, even inside a real request.

Both `am.exec(ctx)` and `am.exec_without_ctx(db)` also record a `History` row per write for any model with `#[model(history = true)]` - `exec_without_ctx` only skips the `*_by_id` fields, it does not skip history. See [History](history.md).
