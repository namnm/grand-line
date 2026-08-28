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

## Model attributes

Placed directly **under** `#[model]`, each one implements a framework trait on the entity so your model can back a built-in DI impl without any hand-written lookup code. They require the `auth`/`authz` feature and a few specific field names, and error at compile time naming the field when one is missing.

| Attribute               | Backs                                  | Docs                                                |
| ----------------------- | -------------------------------------- | --------------------------------------------------- |
| `#[auth_session]`       | the default `AuthSessionImpl`          | [Authentication](authentication.md#setup)           |
| `#[auth_otp]`           | the default `AuthOtpImpl`              | [Authentication](authentication.md#setup)           |
| `#[authz_org]`          | the default `AuthzOrgImpl`             | [Authorization](authorization.md#setup)             |
| `#[authz_role]`         | the default `AuthzRoleImpl`            | [Authorization](authorization.md#setup)             |
| `#[authz_user_in_role]` | the role lookup's user assignment side | [Authorization](authorization.md#setup)             |
| `#[authz_org_id]`       | the `ctx.authz_org_*` scoping helpers  | [Authorization](authorization.md#org-scoped-models) |

```rs
#[model(deleted_at = false, by_id = false)]
#[auth_session]
pub struct LoginSession {
    pub user_id: String,
    #[graphql(skip)]
    pub secret_hashed: String,
}
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

**`#[graphql(skip)]`** - stored in the DB, hidden from the GraphQL schema. Also dropped from the `History` snapshot (see [History](history.md#what-the-snapshot-stores)).

**`#[history(skip)]`** - stored in the DB and exposed over GraphQL as usual, only left out of the `History` snapshot. For a column that is fine to serve but not worth retaining in an audit trail.

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

### Reading a related row from a resolver

`sql_dep` covers the columns of the model the field is on. To read a row of _another_ model with dataloader batching, use `gql_load_with` and name the columns yourself:

```rs
#[resolver(sql_dep = "org_id")]
pub org_label: String,

async fn resolve_org_label(u: &UserGql, ctx: &Context<'_>) -> Res<String> {
    let id = u.org_id.clone().ok_or(CoreDbErr::GqlResolverNone)?;
    let db = &ctx.db().await?;

    let org = Org::gql_load_with(
        ctx,
        db,
        OrgColumn::Id,
        id,
        None, // authz_row
        None, // include_deleted
        None, // extra filter
        Org::gql_look_ahead_cols(&[OrgColumn::Name, OrgColumn::Slug]),
    )
    .await?
    .ok_or(CoreDbErr::Db404)?;

    org.name.ok_or(CoreDbErr::GqlResolverNone.into())
}
```

The plain `gql_load` picks its columns from the calling field's own GraphQL selection set. A scalar resolver has no selection set of its own, so `gql_load` cannot know what to select there and returns an error naming `gql_load_with` - it does not silently hand back a row of `None`s. Use `gql_load` only from a relation field.

`Org::gql_look_ahead_all()` loads every column reachable over GraphQL when the column list would be tedious. It also evaluates every `#[sql_expr]` the model has, so prefer naming columns when that matters. `#[graphql(skip)]` columns are never included by either.

Don't reach for a plain `Org::find().one(db)` here - it works, but it runs one query per parent row instead of one batched query per request.

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
