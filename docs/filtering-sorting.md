# Filtering and sorting

```rs
let f = filter!(Todo {
    done: true,
    content_starts_with: "2024",
});
let f = TodoFilter::combine_and(f1, f2);

let sort = order_by!(Todo [CreatedAtDesc, ContentAsc]);
```

Filter operators per column (`content: String`):

```
content  content_eq  content_ne  content_in  content_not_in
content_gt  content_gte  content_lt  content_lte
content_like  content_starts_with  content_ends_with
```

`TodoFilter` also has top-level `AND`, `OR`, `NOT` (uppercase in GraphQL, `and`/`or`/`not` on the Rust struct) for nested conditions:

```graphql
{ AND: [{ done: true }, { content_starts_with: "2024" }] }
{ OR: [{ content_starts_with: "2024" }, { content_starts_with: "2023" }] }
{ NOT: { done: true } }
```

On a model with `deleted_at`, referencing `deletedAt`/`deletedAt_ne` anywhere in that tree (even inside an `AND`/`OR`/`NOT` branch) is treated as asking to see soft-deleted rows - by design, not just at the top level. See [CRUD resolvers](crud-resolvers.md) for how this composes with the `include_deleted` resolver attribute.

## Request limits

`CoreConfig` bounds what a single request may ask for. Attach it to the schema to change the defaults:

```rs
GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
    .extension(GrandLineExtension)
    .data(Arc::new(db.clone()))
    .data(CoreConfig {
        limit_default: 10,
        limit_max: 100,
        offset_max: 10_000,
        order_by_max: 5,
    })
    .finish()
```

| Field           | Default  | Bounds                                        |
| --------------- | -------- | --------------------------------------------- |
| `limit_default` | `10`     | rows returned when `page.limit` is omitted    |
| `limit_max`     | `100`    | `page.limit` is clamped to it                 |
| `offset_max`    | `10_000` | `page.offset` is clamped to it                |
| `order_by_max`  | `5`      | how many `order_by` entries a client may send |

`offset_max` matters more than it looks: a deep offset is a full scan the database cannot shortcut, reachable from one ordinary looking query. `order_by_max` caps only the client supplied list - a resolver's own default `order_by` is deliberate and is never truncated.
