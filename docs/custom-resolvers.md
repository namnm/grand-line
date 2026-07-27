# Custom resolvers

```rs
#[query]
fn todo_count_done() -> u64 {
    filter!(Todo {
        done: true,
    })
    .into_select()
    .count(tx)
    .await?
}

#[mutation]
fn todo_delete_done() -> Vec<TodoGql> {
    let f = filter!(Todo {
        done: true,
    });
    Todo::soft_delete_many()?
        .filter(f.clone())
        .exec(tx)
        .await?;
    f.gql_select_id().all(tx).await?
}
```

`f.gql_select_id()` only loads the `id` column - a `Vec<TodoGql>` built this way only supports requesting `id` from the client, any other field (e.g. `content`) errors as an unresolved value. Use `f.gql_select(ctx)?` instead when the response needs to expose more than the id.

These generate `TodoCountDoneQuery` / `TodoDeleteDoneMutation` structs for use in `MergedObject` (see [Schema collector](schema-collector.md) to avoid listing them by hand).
