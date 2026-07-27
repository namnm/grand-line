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
