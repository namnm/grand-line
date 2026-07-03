# GrandLine

Rust macro framework for building GraphQL APIs on top of `sea-orm` and `async-graphql` - automatic CRUD resolvers, nested filtering, sorting, pagination, relationships, and soft-delete.

<p align="center">
  <img src="https://github.com/nongdan-dev/grand-line/blob/master/.md/banner.jpg?raw=true" alt="Grand Line One Piece"/>
</p>

- [Simple Todo example](https://github.com/nongdan-dev/grand-line/blob/master/examples/simple_todo/src/main.rs)
- [All examples](https://github.com/nongdan-dev/grand-line/blob/master/examples)
- [Tests](https://github.com/nongdan-dev/grand-line/blob/master/tests)

---

### Contents

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->

- [Quick start](#quick-start)
- [Model](#model)
  - [Auto-generated types](#auto-generated-types)
  - [Auto-added fields](#auto-added-fields)
  - [Field attributes](#field-attributes)
  - [Input types and enums](#input-types-and-enums)
- [CRUD resolvers](#crud-resolvers)
- [Custom resolvers](#custom-resolvers)
- [Relationships](#relationships)
  - [Custom relation resolvers](#custom-relation-resolvers)
- [Filtering and sorting](#filtering-and-sorting)
- [Schema collector](#schema-collector)
- [Resolver bodies](#resolver-bodies)
- [Context](#context)
- [Transactions](#transactions)
- [Active model helpers](#active-model-helpers)
- [SeaORM query helpers](#seaorm-query-helpers)
- [Error handling](#error-handling)
- [Authentication](#authentication)
  - [Setup](#setup)
  - [Defining your User model](#defining-your-user-model)
  - [Register](#register)
  - [Login](#login)
  - [Forgot password](#forgot-password)
  - [Session management](#session-management)
  - [`auth` attribute](#auth-attribute)
  - [Customizing behavior](#customizing-behavior)
- [Authorization](#authorization)
  - [Setup](#setup-1)
  - [Defining your Org model](#defining-your-org-model)
  - [`authz` attribute](#authz-attribute)
  - [Col policy structure](#col-policy-structure)
  - [Row policy and `authz_row`](#row-policy-and-authz_row)
- [Debug macro outputs](#debug-macro-outputs)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

---

### Quick start

```rs
use grand_line::prelude::*;

#[model]
pub struct Todo {
    pub content: String,
    pub done: bool,
}

#[search(Todo)]
fn resolver() {
}

#[gql_input]
pub struct TodoCreate {
    pub content: String,
}
#[create(Todo)]
fn resolver() {
    am_create!(Todo {
        content: data.content,
    })
}
```

<p align="center">
  <img src="https://github.com/nongdan-dev/grand-line/blob/master/.md/altair.jpg?raw=true" alt="Altair screenshot"/>
</p>

That produces a `todoSearch` query with filter/sort/pagination, and a `todoCreate` mutation - all type-safe, all wired to the database.

---

### Model

#### Auto-generated types

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

#### Auto-added fields

Every model gets these automatically:

| Field           | Type                    | Set on       |
| --------------- | ----------------------- | ------------ |
| `id`            | `String` (26-char ULID) | insert       |
| `created_at`    | `DateTimeUtc`           | insert       |
| `updated_at`    | `DateTimeUtc`           | every update |
| `deleted_at`    | `Option<DateTimeUtc>`   | soft-delete  |
| `created_by_id` | `Option<String>`        | manually     |
| `updated_by_id` | `Option<String>`        | manually     |
| `deleted_by_id` | `Option<String>`        | manually     |

Opt out per model:

```rs
#[model(created_at = false)] // no created_at / created_by_id
#[model(updated_at = false)] // no updated_at / updated_by_id
#[model(deleted_at = false)] // no deleted_at / deleted_by_id - also disables soft-delete
#[model(by_id = false)]      // no *_by_id fields
```

#### Field attributes

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

**`#[graphql(skip)]`** - stored in the DB, hidden from the GraphQL schema.

**`#[sql_expr(...)]`** - GraphQL-only computed column, evaluated by the database:

```rs
#[sql_expr(Expr::col(Column::Price).mul(Expr::val(1.0).sub(Expr::col(Column::DiscountPercentage).div(100.0))))]
pub discounted_price: f64,
```

**`#[resolver(sql_dep = "col1, col2")]`** - GraphQL-only field resolved in Rust. Requires a `resolve_{field_name}` function in the same scope, or write it with the `#[field_resolver]` macro (see [Custom relation resolvers](#custom-relation-resolvers)):

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

#### Input types and enums

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

---

### CRUD resolvers

When the function is named `resolver`, the GraphQL field defaults to `{model}{Operation}` (e.g. `todoSearch`). Any other name overrides it.

The input type for `#[create]` / `#[update]` is the PascalCase of the GraphQL field name.

| Macro       | Body returns                                    | Injected locals                                 | Output              |
| ----------- | ----------------------------------------------- | ----------------------------------------------- | ------------------- |
| `#[search]` | `Search<TodoOrderBy>`                           | `filter`, `order_by`, `page`, `include_deleted` | `Vec<TodoGql>`      |
| `#[count]`  | `Count`                                         | `filter`, `include_deleted`                     | `u64`               |
| `#[detail]` | `Detail`                                        | `id`, `include_deleted`                         | `Option<TodoGql>`   |
| `#[create]` | `ActiveModelWrapper<AmCreate, TodoActiveModel>` | `data: TodoCreate`                              | `TodoGql`           |
| `#[update]` | `ActiveModelWrapper<AmUpdate, TodoActiveModel>` | `id`, `data: TodoUpdate`                        | `TodoGql`           |
| `#[delete]` | nothing (pre-delete hook)                       | `id`, `permanent: Option<bool>`                 | `TodoGql` (id only) |

`am_create!`/`am_update!` (see [Active model helpers](#active-model-helpers)) already produce the `ActiveModelWrapper` type `#[create]`/`#[update]` expect - you rarely need to name it directly.

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
    Todo::find_by_id(&id).exists_or_404(tx).await?;
    am_update!(Todo {
        id: id.clone(),
        content: data.content,
    })
}

#[delete(Todo)]
fn resolver() {
    Todo::find_by_id(&id).exists_or_404(tx).await?;
}

#[delete(Todo, permanent_delete = false)] // remove the permanent option
fn resolver() {
}
```

Use `resolver_inputs` to define fully custom parameters instead of the ones the table above injects:

```rs
#[update(Todo, resolver_inputs)]
fn todo_toggle_done(id: String) {
    let todo = Todo::find_by_id(&id).one_or_404(tx).await?;
    am_update!(Todo {
        id: id.clone(),
        done: !todo.done,
    })
}
```

---

### Custom resolvers

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

These generate `TodoCountDoneQuery` / `TodoDeleteDoneMutation` structs for use in `MergedObject` (see [Schema collector](#schema-collector) to avoid listing them by hand).

---

### Relationships

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

#### Custom relation resolvers

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

`#[resolver(sql_dep = "...")]` plain field resolvers (see [Field attributes](#field-attributes)) can use the same generator pattern via `#[field_resolver]` - it reads the output type straight from your function signature and wraps it for you:

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

---

### Filtering and sorting

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

`TodoFilter` also has top-level `and`, `or`, `not` for nested conditions.

---

### Schema collector

Each resolver macro generates a named struct (`TodoSearchQuery`, `TodoCreateMutation`, etc.). Normally you must list all of them manually in a `MergedObject`:

```rs
// Manual - must add each resolver type by hand
#[derive(Default, MergedObject)]
struct Query(TodoSearchQuery, TodoCountQuery, TodoDetailQuery, TodoCountDoneQuery);

#[derive(Default, MergedObject)]
struct Mutation(
    TodoCreateMutation,
    TodoUpdateMutation,
    TodoDeleteMutation,
    TodoDeleteDoneMutation,
);
```

`grand_line_build` eliminates this by scanning source files at build time and auto-generating `Query` and `Mutation`. It works across crates - any source directory can be included.

Add it as a build dependency:

```toml
[build-dependencies]
grand_line_build = { path = "../packages/grand_line_build" }
```

Create or edit `build.rs` at the crate root:

```rs
```

This scans `src/` of the current crate. Then include the generated file in your crate root:

```rs
grand_line::include_generated_schema! {}

fn schema(db: &DatabaseConnection) -> GraphQLSchema<Query, Mutation, EmptySubscription> {
    GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
        .extension(GrandLineExtension)
        .data(Arc::new(db.clone()))
        .finish()
}
```

For more control - multiple source directories and external merged types (e.g. from auth):

```rs
fn main() {
    grand_line_build::SchemaBuilder::new()
        .scan("src")
        .scan("../other_crate/src")
        .extra_query("AuthMergedQuery")
        .extra_mutation("AuthMergedMutation<User>")
        .generate();
}
```

The generated `Query` and `Mutation` match the names produced by the resolver macros exactly (same naming convention). `rerun-if-changed` directives are emitted automatically for each scanned directory.

---

### Resolver bodies

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

**Caveat:** this check is syntactic (does the last statement lack a trailing `;`), not a type check. A stray trailing `;` after what was meant to be the tail expression silently discards it and appends `Default::default()` instead of failing to compile - double-check you haven't left a `;` on your last line in these bodies. This only affects the _outermost_ statement position: an `if`/`match` used correctly as the tail is unaffected, and if a branch of that `if`/`match` ends in `;` instead of an expression, the compiler still catches the resulting type mismatch normally (the macro doesn't reach into nested blocks to paper over it).

---

### Context

`ctx` is injected into every resolver. Key methods:

**Core**

```rs
ctx.tx().await?                       // Arc<DatabaseTransaction>
ctx.cache(|| async { ... }).await?    // Arc<T> - per-request memoize by type
```

**Auth (`grand_line_auth`)**

| Method                                       | Returns                        | Description                                                      |
| -------------------------------------------- | ------------------------------ | ---------------------------------------------------------------- |
| `ctx.auth().await?`                          | `String`                       | Current user's `id`; errors with `Unauthenticated` if no session |
| `ctx.auth_unchecked().await?`                | `Arc<Option<LoginSessionSql>>` | Current session or `None`                                        |
| `ctx.auth_ensure_authenticated().await?`     | `()`                           | Errors if no session                                             |
| `ctx.auth_ensure_not_authenticated().await?` | `()`                           | Errors if already logged in                                      |

**Authz (`grand_line_authz`)**

| Method                        | Returns           | Description                                          |
| ----------------------------- | ----------------- | ---------------------------------------------------- |
| `ctx.authz().await?`          | `String`          | Verified `org_id` from `X-Org-Id` header             |
| `ctx.authz_role().await?`     | `RoleSql`         | The matched `Role` row                               |
| `ctx.authz_row::<F>().await?` | `Option<F>`       | Row-level filter from the role's `row_policy` script |
| `ctx.org_unchecked().await?`  | `Arc<OrgMinimal>` | Org from `X-Org-Id` without an auth check            |

---

### Transactions

`GrandLineExtension` manages one lazy transaction per request - commits on success, rolls back on any error.

```rs
GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
    .extension(GrandLineExtension)
    .data(Arc::new(db.clone()))
    .finish()
```

**Known limitation:** all resolvers in a request share one `DatabaseTransaction`, i.e. one underlying DB connection. Sibling GraphQL fields (including sibling relation resolvers) may be scheduled concurrently as Rust futures, but their SQL statements still serialize one at a time on that connection - there is no query-level parallelism within a request today. Giving read-only requests their own pooled connections (instead of one shared transaction) would let sibling relations actually run in parallel; this is not implemented yet. Mutations would keep the single-transaction model for write consistency.

---

### Active model helpers

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
am.exec_without_ctx(tx).await?;     // insert/update, no by_id fields
am.into_active_model(ctx).await?;   // unwrap to raw sea-orm ActiveModel, sets *_by_id from ctx.auth()
am.into_active_model_without_ctx(); // unwrap to raw sea-orm ActiveModel, no by_id fields

Todo::soft_delete_by_id(&id)?.exec(tx).await?;
Todo::soft_delete_many()?.filter(condition).exec(tx).await?;
am.soft_delete(tx).await?; // on an active model instance
```

---

### SeaORM query helpers

The traits in `packages/core/db` extend sea-orm's `Select`, `Filter`, and `ActiveModel` types with convenience methods available throughout resolvers.

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

---

### Error handling

```rs
#[grand_line_err]
enum MyErr {
    #[error("record not found")]
    #[client] // forwarded to the response as-is
    NotFound,

    #[error("oops")] // client sees a generic "internal server error"
    InternalProblem,
}

// Raise from any resolver:
Err(MyErr::NotFound)?;

// Downcast from a response error:
error.source
    .as_deref()
    .and_then(|e| e.downcast_ref::<GrandLineErr>())
    .map(|e| e.0.code()); // e.g. "NotFound"
```

Any error that isn't `#[client]` is logged to stderr with its real message and replaced with a generic internal-server error before it reaches the client, so accidental leaks of internal detail are opt-in, not opt-out.

---

### Authentication

`grand_line_auth` provides email + password auth with OTP for register and forgot-password.

#### Setup

Define your `User` model (see [next section](#defining-your-user-model)), then wire up the schema:

```rs
#[derive(Default, MergedObject)]
pub struct Query(AuthMergedQuery, /* your queries */);

#[derive(Default, MergedObject)]
pub struct Mutation(AuthMergedMutation<User>, /* your mutations */);

GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
    .extension(GrandLineExtension)
    .data(Arc::new(db.clone()))
    .data(AuthConfig::default())
    .finish()
```

Your migration must include the `User` table (defined by you) plus `AuthOtp` and `LoginSession` (provided by the framework).

#### Defining your User model

The framework does not ship a `User` model. You define your own and implement `AuthUser`:

```rs
#[model(by_id = false)]
pub struct User {
    pub email: String,
    #[graphql(skip)]
    pub password_hashed: String,
    // add any fields you need:
    pub display_name: String,
    pub avatar_url: Option<String>,
}

impl AuthUser for User {
    fn email_col() -> UserColumn {
        UserColumn::Email
    }
    fn hashed_password_col() -> UserColumn {
        UserColumn::PasswordHashed
    }
    fn get_email(m: &UserSql) -> &str {
        &m.email
    }
    fn get_password_hashed(m: &UserSql) -> &str {
        &m.password_hashed
    }
}
```

The framework reads and writes `User` exclusively through these four methods, so the rest of your model is yours to define freely.

#### Register

Two-step OTP flow:

```graphql
# Step 1 - triggers on_otp_create (send OTP by email)
mutation {
    register(data: { email: "user@example.com", password: "Str0ngP@ssw0rd?" }) {
        secret # save this - needed in step 2
    }
}

# Step 2
mutation {
    registerResolve(data: { id: "...", secret: "...", otp: "123456" }) {
        secret # bearer token for subsequent requests
        inner { userId }
    }
}
```

#### Login

```graphql
mutation {
    login(data: { email: "user@example.com", password: "123123" }) {
        secret
        inner { userId }
    }
}
```

Send the token on subsequent requests: `Authorization: Bearer {secret}`

#### Forgot password

```graphql
# Step 1
mutation { forgot(data: { email: "user@example.com" }) { secret } }

# Step 2
mutation {
    forgotResolve(data: { id: "...", secret: "...", otp: "123456" }, password: "NewP@ssw0rd!") {
        secret
        inner { userId }
    }
}
```

#### Session management

```graphql
query  { loginSessionCurrent { userId ip } }
query  { loginSessionSearch { userId ip ua } }
query  { loginSessionCount }
mutation { loginSessionDelete(id: "...") { id } }
mutation { loginSessionDeleteAll }
mutation { logout { id } }
```

#### `auth` attribute

```rs
#[query(auth)]
fn my_profile() -> UserGql {
}

#[mutation(auth(unauthenticated))]
fn register() -> AuthOtpWithSecret {
}

#[search(Todo, auth)]
fn resolver() {
}
```

#### Customizing behavior

Implement `AuthHandlers` to hook into OTP delivery, password validation, and post-auth lifecycle events - every method has a no-op default, override only what you need:

```rs
struct MyHandlers;

#[async_trait]
impl AuthHandlers for MyHandlers {
    async fn otp(&self, _ctx: &Context<'_>) -> Res<String> {
        Ok(generate_otp()) // custom OTP generator, defaults to a random 6-digit code
    }
    async fn on_otp_create(&self, _ctx: &Context<'_>, otp: &AuthOtpSql, raw: &str) -> Res<()> {
        send_email(&otp.email, raw).await // send the OTP by email
    }
    async fn on_register_resolve(&self, ctx: &Context<'_>, user_id: &str, _ls: &LoginSessionSql) -> Res<()> {
        let tx = &*ctx.tx().await?;
        am_create!(UserProfile {
            user_id: user_id.to_owned(),
            bio: "",
        })
        .exec(ctx)
        .await?;
        Ok(())
    }
    // also available: password_validate, on_login_resolve, on_forgot_resolve
}

AuthConfig {
    handlers: Arc::new(MyHandlers),
    ..Default::default()
}
```

---

### Authorization

`grand_line_authz` provides role-based access control with org scoping, field-level policy checks, and row-level filtering.

#### Setup

Define your `Org` model (see [next section](#defining-your-org-model)), then wire up the schema:

```rs
GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
    .extension(GrandLineExtension)
    .data(Arc::new(db.clone()))
    .data(AuthConfig::default())
    .data(AuthzConfig::default())
    .data(authz_org_impl::<Org>())
    .finish()
```

Your migration must include the `User` and `Org` tables (defined by you) plus `LoginSession`, `AuthOtp` (auth), `Role`, and `UserInRole` (authz).

Roles use a `realm` string to categorize scope. Common patterns:

| realm    | Attribute                                         | Checks     |
| -------- | ------------------------------------------------- | ---------- |
| `org`    | `#[authz(realm = "org")]`                         | user + org |
| `system` | `#[authz(realm = "system", skip_org)]`            | user only  |
| `public` | `#[authz(realm = "public", skip_user, skip_org)]` | none       |

Each request must include the `X-Role-Id` header with the UUID of the role being used. The framework verifies the role exists, matches the required realm and org, and that the authenticated user is listed in `UserInRole` for that role.

```rs
am_create!(Role {
    name: "Org Admin", realm: "org",
    col_policy: col_policy.to_json()?,
    row_policy: RowPolicy::default().to_json()?,
    org_id: Some(org_id.clone()),
})
.exec(ctx).await?;

am_create!(UserInRole {
    user_id: user_id.clone(), role_id: role_id.clone(),
    org_id: Some(org_id.clone()), // must match the role's org_id
})
.exec(ctx).await?;
```

By default a mismatch (missing role, wrong org, wrong realm) surfaces as `AuthzErr::Unauthorized`; set `AuthzConfig.unauthorized_err` to swap that for something like `CoreDbErr::Db404` if you don't want to reveal that a resource exists.

#### Defining your Org model

The framework does not ship an `Org` model. Define your own and implement `AuthzOrg`:

```rs
#[model]
pub struct Org {
    pub name: String,
    // add any fields you need:
    pub logo_url: Option<String>,
    pub plan: OrgPlan,
}

// marker trait - EntityX provides everything needed
impl AuthzOrg for Org {}
```

The framework looks up orgs via `authz_org_impl::<Org>()` using the `id` from the `X-Org-Id` header. Your custom fields are accessible in your own resolvers via normal `Org::find()` queries.

#### `authz` attribute

```rs
// Org-scoped: requires Authorization + X-Org-Id + X-Role-Id headers
#[query(authz(realm = "org"))]
fn org_dashboard() -> OrgGql {
    let org_id = ctx.authz().await?;
    Org::find_by_id(&org_id).gql_select(ctx)?.one_or_404(tx).await?
}

// System-wide: requires Authorization + X-Role-Id (no X-Org-Id)
#[query(authz(realm = "system", skip_org))]
fn system_dashboard() -> String {
    "ok".to_string()
}

// Works on all CRUD macros - use authz_row for row-level filtering
#[search(Task, authz(realm = "org"))]
fn resolver() {
    ctx.authz_row::<TaskFilter>().await?.into()
}
```

Use `ctx.authz_role().await?` to get the matched `Role` row inside any authz-guarded resolver.

#### Col policy structure

`Role.col_policy` is a JSON-encoded `ColPolicy` map that controls which GraphQL operations and fields are allowed:

```rs
pub type ColPolicy = HashMap<String, ColPolicyOperation>;

pub struct ColPolicyOperation {
    pub inputs: ColPolicyField, // allowed GraphQL arguments
    pub output: ColPolicyField, // allowed response fields
}

pub struct ColPolicyField {
    pub allow: bool,
    pub children: Option<ColPolicyFields>, // HashMap<String, ColPolicyField>
}
```

Key is the GraphQL operation name, or `"*"` for all. Wildcards in children:

| Key    | Meaning                            |
| ------ | ---------------------------------- |
| `"*"`  | Allow any direct child field       |
| `"**"` | Allow any nested field recursively |

**Allow everything:**

```rs
let all = ColPolicyField {
    allow: true,
    children: Some(hashmap! {
        "**".to_owned() => ColPolicyField {
            allow: true,
            children: None,
        },
    }),
};
let col: ColPolicy = hashmap! {
    "*".to_owned() => ColPolicyOperation {
        inputs: all.clone(),
        output: all,
    },
};
```

**Restrict to specific fields:**

```rs
let col: ColPolicy = hashmap! {
    "taskSearch".to_owned() => ColPolicyOperation {
        inputs: ColPolicyField {
            allow: true,
            children: Some(hashmap! {
                "filter".to_owned() => ColPolicyField {
                    allow: true,
                    children: Some(hashmap! {
                        "**".to_owned() => ColPolicyField {
                            allow: true,
                            children: None,
                        },
                    }),
                },
            }),
        },
        output: ColPolicyField {
            allow: true,
            children: Some(hashmap! {
                "id".to_owned() => ColPolicyField { allow: true, children: None },
                "title".to_owned() => ColPolicyField { allow: true, children: None },
            }),
        },
    },
};
```

#### Row policy and `authz_row`

`Role.row_policy` is a JSON-encoded `RowPolicy` map from field path to a script string. The script is forwarded verbatim to `AuthzHandlers::execute_script` so your app can produce a filter for that resolver.

Key is the GraphQL field path (e.g. `"tasks"` or `"users.posts"`). Value is an arbitrary string - the framework passes it to `execute_script` unchanged.

```rs
let row: RowPolicy = hashmap! {
    "tasks".to_owned() => "filter_by_assignee".to_owned(),
};
am_create!(Role {
    col_policy: col.to_json()?,
    row_policy: row.to_json()?,
})
```

Inside a resolver, call `ctx.authz_row::<F>()`. Authorization is already guaranteed by the macro. Returns `None` when no entry exists for this field (all rows accessible), or `Some(F)` when the script produced a filter:

```rs
#[search(Task, authz(realm = "org"))]
fn resolver() {
    ctx.authz_row::<TaskFilter>().await?.into()
}
```

Implement `AuthzHandlers` to evaluate the script and return a JSON object that deserializes into your filter type:

```rs
struct MyHandlers;

#[async_trait]
impl AuthzHandlers for MyHandlers {
    async fn execute_script(&self, ctx: &Context<'_>, script: &str) -> Res<Option<JsonValue>> {
        let user_id = ctx.auth().await?;
        let org_id = ctx.authz().await?;
        // evaluate script (Rhai, hand-written match, etc.)
        Ok(Some(json!({
            "assignee_id_eq": user_id,
            "org_id_eq": org_id,
        })))
    }
}

AuthzConfig {
    handlers: Arc::new(MyHandlers),
    ..Default::default()
}
```

`authz_row` results and field-path resolution are cached per request, and alias-aware - `myTasks: tasks { ... }` still resolves the row policy for the real field `tasks`, so aliasing a query can't be used to bypass a row policy.

---

### Debug macro outputs

Set `DEBUG_MACRO=1` and enable a feature flag:

- `debug_macro_cli` - prints generated code to stdout during build
- `debug_macro_file` - writes generated code to `target/grand-line/` during build
