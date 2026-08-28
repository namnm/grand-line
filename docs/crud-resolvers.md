# CRUD resolvers

When the function is named `resolver`, the GraphQL field defaults to `{model}{Operation}` (e.g. `todoSearch`). Any other name overrides it.

The input type for `#[create]` / `#[update]` is the PascalCase of the GraphQL field name.

| Macro       | Body returns                           | Injected locals                                 | Output              |
| ----------- | -------------------------------------- | ----------------------------------------------- | ------------------- |
| `#[search]` | `Search<TodoOrderBy>`                  | `filter`, `order_by`, `page`, `include_deleted` | `Vec<TodoGql>`      |
| `#[count]`  | `Count`                                | `filter`, `include_deleted`                     | `u64`               |
| `#[detail]` | `Detail`                               | `id`, `include_deleted`                         | `Option<TodoGql>`   |
| `#[create]` | `AmWrapper<AmCreate, TodoActiveModel>` | `data: TodoCreate`                              | `TodoGql`           |
| `#[update]` | `AmWrapper<AmUpdate, TodoActiveModel>` | `id`, `data: TodoUpdate`                        | `TodoGql`           |
| `#[delete]` | nothing (pre-delete hook)              | `id`, `permanent: Option<bool>`                 | `TodoGql` (id only) |

`am_create!`/`am_update!` (see [Active model helpers](active-model-helpers.md)) already produce the `AmWrapper` type `#[create]`/`#[update]` expect - you rarely need to name it directly.

`#[search]`/`#[count]`/`#[detail]` expose the `include_deleted` GraphQL argument by default (gated by the `resolver_include_deleted` feature flag). Turn it off per resolver to stop clients from ever passing it:

```rs
#[search(Todo, include_deleted = false)] // no includeDeleted argument in the schema for this resolver
fn resolver() {
}
```

A client filter that itself references `deletedAt`/`deletedAt_ne` (anywhere in the filter, including a nested `and`/`or`/`not` branch, see [Filtering and sorting](filtering-sorting.md)) also opts that query into seeing deleted rows, independently of the `include_deleted` argument above - by design, filtering on deletion state at all is treated as asking to see deleted rows.

`Search<O>` / `Count` / `Detail` are what a resolver returns to add extra deleted-visibility/condition/ordering on top of what the client sent:

```rs
pub struct Filter {
    pub include_deleted: bool, // used if the client didn't pass includeDeleted
    pub condition: Condition,  // will be AND-ed with the client filter
}
pub struct Search<O>
where
    O: OrderBy,
{
    pub filter: Filter,
    pub default_order_by: Vec<O>, // used if the client didn't request an order by
}
pub type Count = Filter;
pub type Detail = Filter;
```

Build one from a filter (`include_deleted` is inherited from the filter's own `deletedAt`/`deletedAt_ne` if the client filtered on it):

```rs
#[search(Todo)]
fn resolver() {
    let extra_filter = filter!(Todo {
        content_starts_with: "2024",
    });
    let default_order_by = order_by!(Todo[CreatedAtDesc]);
    (extra_filter, default_order_by).into()
}

#[create(Todo)]
fn resolver() {
    am_create!(Todo {
        content: data.content,
    })
}

#[update(Todo)]
fn resolver() {
    Todo::find_by_id(&id).exists_or_404(db).await?;
    am_update!(Todo {
        id: id.clone(),
        content: data.content,
    })
}

#[delete(Todo)]
fn resolver() {
    Todo::find_by_id(&id).exists_or_404(db).await?;
}

#[delete(Todo, permanent = false)] // remove the permanent option
fn resolver() {
}
```

Use `resolver_inputs` to define fully custom parameters instead of the ones the table above injects:

```rs
#[update(Todo, resolver_inputs)]
fn todo_toggle_done(id: String) {
    let todo = Todo::find_by_id(&id).one_or_404(db).await?;
    am_update!(Todo {
        id: id.clone(),
        done: !todo.done,
    })
}
```
