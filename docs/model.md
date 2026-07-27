# Model

## Auto-generated types

`#[model]` on `struct Todo` generates:

| Type              | Description                                       |
| ----------------- | ------------------------------------------------- |
| `Todo`            | sea-orm `Entity`                                  |
| `TodoSql`         | sea-orm `Model`                                   |
| `TodoColumn`      | sea-orm `Column`                                  |
| `TodoActiveModel` | sea-orm `ActiveModel`                             |
| `TodoGql`         | async-graphql output object (named `Todo` in GQL) |
| `TodoFilter`      | async-graphql filter input                        |
| `TodoOrderBy`     | async-graphql order by enum                       |

## Auto-added fields

Every model gets these automatically:

| Field           | Type                    | Set on             |
| --------------- | ----------------------- | ------------------ |
| `id`            | `String` (26-char ULID) | insert             |
| `created_at`    | `DateTimeUtc`           | insert             |
| `updated_at`    | `DateTimeUtc`           | every update       |
| `deleted_at`    | `Option<DateTimeUtc>`   | soft-delete        |
| `created_by_id` | `Option<String>`        | using am.exec(ctx) |
| `updated_by_id` | `Option<String>`        | using am.exec(ctx) |
| `deleted_by_id` | `Option<String>`        | using am.exec(ctx) |

Opt out per model:

```rs
#[model(created_at = false)] // no created_at / created_by_id
#[model(updated_at = false)] // no updated_at / updated_by_id
#[model(deleted_at = false)] // no deleted_at / deleted_by_id - also disables soft-delete
#[model(by_id = false)]      // no *_by_id fields
```

## Field attributes

**`#[default(...)]`** - applied at insert when the field is omitted from `am_create!`:

```rs
#[model]
pub struct Todo {
    pub content: String,
    #[default(false)]
    pub done: bool,
    #[default(days_from_now(7))] // any valid Rust expression
    pub due_at: DateTimeUtc,
}
```

A `bool`/numeric field with no exposed create input (see [CRUD resolvers](crud-resolvers.md)) still needs a `#[default(...)]` if nothing ever sets it explicitly - the underlying column has no default of its own otherwise, and an insert that omits it fails with a NOT NULL constraint error.

**`#[graphql(skip)]`** - stored in the DB, hidden from the GraphQL schema.

**`#[sql_expr(...)]`** - GraphQL-only computed column, evaluated by the database:

```rs
#[sql_expr(Expr::col(Column::Price).mul(Expr::val(1.0).sub(Expr::col(Column::DiscountPercentage).div(100.0))))]
pub discounted_price: f64,
```

**`#[resolver(sql_dep = "col1, col2")]`** - GraphQL-only field resolved in Rust. Requires a `resolve_{field_name}` function in the same scope, or write it with the `#[field_resolver]` macro (see [Custom relation resolvers](relationships.md#custom-relation-resolvers)):

```rs
#[resolver(sql_dep = "first_name, last_name")]
pub full_name: String,

async fn resolve_full_name(u: &UserGql, _ctx: &Context<'_>) -> Res<String> {
    let first_name = u.first_name.clone().ok_or(CoreDbErr::GqlResolverNone)?;
    let last_name = u.last_name.clone().ok_or(CoreDbErr::GqlResolverNone)?;
    Ok(format!("{first_name} {last_name}"))
}
```

`sql_dep` lists which underlying SQL columns must be selected for this resolver to run - the framework only fetches columns actually requested in the GraphQL selection, so any column the Rust function reads has to be declared here.

**Performance note:** requesting a `#[sql_expr]` or `#[resolver(sql_dep = ...)]` field from a `#[create]`/`#[update]` response triggers a second `SELECT ... WHERE id = ?` to refetch the row with those columns, on top of the insert/update itself - the write statement itself doesn't return computed/virtual columns. Not a bug, just worth knowing when you see an unexpected extra query in a mutation.

## Input types and enums

```rs
#[gql_input]
pub struct TodoCreate {
    pub content: String,
}

#[gql_enum] // GraphQL-only enum
pub enum Direction {
    Asc,
    Desc,
}

#[sql_enum] // stored as VARCHAR(255) snake_case, exposed in GraphQL
pub enum Status {
    Active,
    Inactive,
}
```
