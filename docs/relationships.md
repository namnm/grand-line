# Relationships

Declare on `#[model]` fields. Resolved with look-ahead - only requested fields are fetched.

```rs
#[model]
pub struct User {
    #[has_one]
    pub profile: UserProfile, // UserProfile holds user_id FK
    #[has_many]
    pub posts: Post,
    #[many_to_many]
    pub orgs: Org, // requires UserInOrg join model
}

#[model]
pub struct Post {
    pub user_id: String,
    #[belongs_to]
    pub user: User,
}

#[model]
pub struct UserInOrg {
    pub user_id: String,
    pub org_id: String,
}
```

`has_one`/`belongs_to` are resolved through a per-request `DataLoader`, batched by the foreign key column - N sibling rows referencing the same `User` fetch that `User` exactly once instead of once per row. `has_many`/`many_to_many` run one query per relation field per parent (no cross-request batching); each relation is still its own query rather than a single SQL JOIN spanning the whole selection tree, so a `User` referenced by 100 `Post` rows never gets duplicated 100 times in the result set the way a JOIN would.

Soft-deleted related records are excluded by default. Override per field:

```graphql
query {
    userDetail(id: "...") {
        profile(includeDeleted: true) {
            bio
        }
        posts(filter: { deletedAt_ne: null }) {
            content
        }
    }
}
```

## Custom relation resolvers

Add `resolver` to scope down what a relation fetches - a generator macro builds the resolver function's full signature for you, so the body only needs to return the extra condition:

| Field attribute                  | On                        | Generator macro                               | Body returns           | Extra injected locals        |
| -------------------------------- | ------------------------- | --------------------------------------------- | ---------------------- | ---------------------------- |
| `resolver` / `resolver = "name"` | `has_many`/`many_to_many` | `#[many_resolver(Model, parent = "Parent")]`  | `Search<ModelOrderBy>` | `filter`, `order_by`, `page` |
| `count`, `count_resolver`        | `has_many`/`many_to_many` | `#[count_resolver(Model, parent = "Parent")]` | `Count`                | `filter`                     |
| `resolver` / `resolver = "name"` | `has_one`/`belongs_to`    | `#[one_resolver(Model, parent = "Parent")]`   | `Option<ModelFilter>`  | -                            |

All three (plus `ctx`, `tx`, `include_deleted`) are auto-injected - the tagged function must take no parameters and declare no return type, both are generated. `parent` is optional; omit it and the function becomes generic over any `GqlModel`, useful when the body doesn't need to read the parent row's own fields.

```rs
#[model]
pub struct User {
    #[has_many(resolver = "recent_posts", count, count_resolver = "published_count")]
    pub posts: Post,
    #[has_one(resolver = "primary_profile")]
    pub profile: UserProfile,
}

#[many_resolver(Post, parent = "User")]
fn recent_posts() {
    order_by!(Post[CreatedAtDesc]).into()
}

#[count_resolver(Post)]
fn published_count() {
    filter!(Post {
        published: true,
    })
    .into()
}

#[one_resolver(UserProfile)]
fn primary_profile() {
    filter!(UserProfile {
        primary: true,
    })
}
```

`has_one`/`belongs_to` resolvers return `Option<ModelFilter>` rather than a raw `Detail`/`Condition` - since these relations are DataLoader-batched (see above), the filter needs to be serializable so it can be folded into the batch key alongside `authz_row`. This keeps two different resolver/authz combinations from ever colliding into the same batch while still batching the common case.

`#[resolver(sql_dep = "...")]` plain field resolvers (see [Field attributes](model.md#field-attributes)) can use the same generator pattern via `#[field_resolver]` - it reads the output type straight from your function signature and wraps it for you:

```rs
#[model]
pub struct AuthOtp {
    #[default(0)]
    #[graphql(skip)]
    pub total_attempt: i64,
    #[resolver(sql_dep = "total_attempt")]
    pub remaining_attempt: i64,
}

#[field_resolver(parent = "AuthOtp")]
fn resolve_remaining_attempt() -> i64 {
    let max = ctx.auth_config().otp_max_attempt;
    max - parent.total_attempt.unwrap_or_default()
}
```
