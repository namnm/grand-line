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
- [File uploads](#file-uploads)
  - [Setup](#setup-2)
  - [Upload chain](#upload-chain)
  - [Client never confirms](#client-never-confirms)
  - [Builtin ffprobe/ffmpeg processing](#builtin-ffprobeffmpeg-processing)
- [Debug macro outputs](#debug-macro-outputs)
- [Design notes](#design-notes)
  - [What GrandLine does well](#what-grandline-does-well)
  - [Known limitations](#known-limitations)
  - [Roadmap](#roadmap)

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

| Macro       | Body returns                           | Injected locals                                 | Output              |
| ----------- | -------------------------------------- | ----------------------------------------------- | ------------------- |
| `#[search]` | `Search<TodoOrderBy>`                  | `filter`, `order_by`, `page`, `include_deleted` | `Vec<TodoGql>`      |
| `#[count]`  | `Count`                                | `filter`, `include_deleted`                     | `u64`               |
| `#[detail]` | `Detail`                               | `id`, `include_deleted`                         | `Option<TodoGql>`   |
| `#[create]` | `AmWrapper<AmCreate, TodoActiveModel>` | `data: TodoCreate`                              | `TodoGql`           |
| `#[update]` | `AmWrapper<AmUpdate, TodoActiveModel>` | `id`, `data: TodoUpdate`                        | `TodoGql`           |
| `#[delete]` | nothing (pre-delete hook)              | `id`, `permanent: Option<bool>`                 | `TodoGql` (id only) |

`am_create!`/`am_update!` (see [Active model helpers](#active-model-helpers)) already produce the `AmWrapper` type `#[create]`/`#[update]` expect - you rarely need to name it directly.

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

#[delete(Todo, permanent = false)] // remove the permanent option
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
am.into_am(ctx).await?;   // unwrap to raw sea-orm ActiveModel, sets *_by_id from ctx.auth()
am.into_am_without_ctx(); // unwrap to raw sea-orm ActiveModel, no by_id fields

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
let col = hashmap! {
    "*".to_owned() => ColPolicyOperation {
        inputs: all.clone(),
        output: all,
    },
};
```

**Restrict to specific fields:**

```rs
let col = hashmap! {
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
let row = hashmap! {
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

### File uploads

`grand_line_file` provides a `File` model backed by an s3/r2-compatible bucket, presigned upload/download urls, and a pluggable processing hook.

#### Setup

```rs
#[derive(Default, MergedObject)]
pub struct Query(FileMergedQuery, /* your queries */);

#[derive(Default, MergedObject)]
pub struct Mutation(FileMergedMutation, /* your mutations */);

GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
    .extension(GrandLineExtension)
    .data(Arc::new(db.clone()))
    .data(FileConfig {
        bucket: "my-bucket".to_owned(),
        ..Default::default()
    })
    .data(Arc::new(s3_client()))
    .finish()
```

The s3 client is not part of `FileConfig`, build it once with your r2 credentials/endpoint and register it the same way as the db connection. R2 is s3-api-compatible but not aws, so the client needs a custom endpoint, a placeholder region, path-style addressing, and the newer aws checksum headers disabled:

```rs
fn s3_client() -> aws_sdk_s3::Client {
    let config = aws_sdk_s3::Config::builder()
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
        .region(aws_sdk_s3::config::Region::new("auto"))
        .endpoint_url("https://<account-id>.r2.cloudflarestorage.com")
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            "<access-key-id>",
            "<secret-access-key>",
            None,
            None,
            "r2",
        ))
        .force_path_style(true)
        .build();
    aws_sdk_s3::Client::from_conf(config)
}
```

Your migration must include the `File` table.

#### Upload chain

```graphql
mutation {
    fileUploadInit(data: { filename: "walter-notes.pdf", contentType: "application/pdf" }) {
        uploadUrl
        inner { id status }
    }
}
```

The client `PUT`s the raw bytes to `uploadUrl` directly, no server involved, then:

```graphql
mutation { fileUploadConfirm(id: "...") { id status size etag downloadUrl } }
```

`fileUploadConfirm` runs a `HeadObject` against the real key (the client-reported size/type from `fileUploadInit` is never trusted), moves the row to `Uploaded`, and calls `FileHandlers::on_upload_confirm`. The default is a no-op - override it to inspect or transform the file (`ffprobe`, image/video compression, thumbnailing...) and move `status` to `Processing`/`Ready`/`Failed` once that work finishes, outside of the request:

```rs
struct MyHandlers;
#[async_trait]
impl FileHandlers for MyHandlers {
    async fn on_upload_confirm(&self, ctx: &Context<'_>, file: &FileSql) -> Res<()> {
        enqueue_processing_job(file.id.clone()); // ffprobe/compress, then update status yourself
        Ok(())
    }
}

FileConfig {
    bucket: "my-bucket".to_owned(),
    handlers: Arc::new(MyHandlers),
    ..Default::default()
}
```

`downloadUrl` is a presigned get url, computed per request, `None` while `status` is still `Pending`.

`fileDelete` only removes the object from the bucket on a permanent delete (`fileDelete(id: "...", permanent: true)`), a soft delete leaves the object in place so it stays recoverable.

`File.orgId` is a plain nullable column, not a `belongs_to` relation, since this package does not own your `Org` model. Add the reverse edge on your own `Org`:

```rs
#[model]
pub struct Org {
    #[has_many]
    pub files: File,
    // ...
}
```

#### Client never confirms

A client can call `fileUploadInit` and then never `PUT` the bytes or never call `fileUploadConfirm` - the row is left `Pending` forever, and if the client did `PUT` without confirming, so is the object. `File.uploadExpiresAt` (`now` + `FileConfig.upload_url_expires_ms`, the same window the presigned put url is valid for) tracks this.

```graphql
mutation { fileCleanupExpiredPending }
```

Finds `Pending` rows whose `uploadExpiresAt` is in the past, best-effort deletes the object (in case one exists despite never being confirmed), and permanently removes the rows, returning the count removed. This package ships no scheduler - call it from your own cron job/worker on whatever interval fits, and gate it with your own auth/authz layer if you don't want it publicly callable, the same way any other custom `#[mutation]` can be wrapped with `authz(realm = "system")`.

#### Builtin ffprobe/ffmpeg processing

`FfmpegFileHandlers` is a ready-to-use `FileHandlers` that shells out to the `ffprobe`/`ffmpeg` binaries - both must be installed on the host system, this crate does not vendor, download, or FFI-bind them, it only runs them as a subprocess, gated behind the `ffmpeg` cargo feature (`file_ffmpeg` at the root) so the `tokio` process/fs features are opt-in:

```rs
FileConfig {
    bucket: "my-bucket".to_owned(),
    handlers: Arc::new(FfmpegFileHandlers::default()),
    ..Default::default()
}

// or with custom tuning
FileConfig {
    handlers: Arc::new(FfmpegFileHandlers(FfmpegConfig {
        max_image_dimension: 1600,
        video_crf: 24,
        ..Default::default()
    })),
    ..Default::default()
}
```

On `fileUploadConfirm`, it flips the row to `Processing` (in the same request/transaction), then in a spawned background task: downloads the object, runs `ffprobe -v quiet -print_format json -show_format -show_streams`, and if `content_type` is `image/*`/`video/*`, downscales/recompresses it with `ffmpeg` - if the result is smaller it is re-uploaded over the same key and `size`/`etag` are updated. The background task uses its own raw db connection rather than `ctx.tx()`, since it outlives the request that triggered it - this is also why `on_upload_confirm` only kicks the work off, it does not await it, a huge video transcode must never block a GraphQL response or hold the shared per-request transaction open.

`ffprobe` failing - binary not installed, non-zero exit, unparsable stdout - never fails the file, it is only there to fill in optional display metadata:

- `File.metadata` (`#[graphql(skip)]`) gets the raw `ffprobe` json, `None` if the probe failed.
- `File.mediaWidth` / `mediaHeight` / `mediaCodec` are read from the first stream with `codec_type: "video"` (a single-frame image is reported by `ffprobe` as one video stream too) so a client can read dimensions/codec without fetching and parsing the whole json blob. All three are `None` if the probe failed or found no video stream (e.g. an audio-only upload).

Minifying, on the other hand, is fatal when attempted - if `ffmpeg` cannot make sense of an `image/*`/`video/*` object, the row moves to `Failed` with the error under `metadata.error`, since that is an actual processing failure, not missing-but-optional metadata. Non-media `content_type`s never attempt minify at all, so a failed probe is the only thing that can happen to them, and the row still reaches `Ready`.

`extract_media_info(probe: &JsonValue) -> MediaInfo { width, height, codec }` is exported standalone if you want the same first-video-stream extraction elsewhere.

Write your own `FileHandlers::on_upload_confirm` instead if you need different tooling (a different image library, virus scanning, a real job queue) - `FfmpegFileHandlers`'s `process`/`mark_failed` helpers are a template for that, not a required base to build on.

---

### Debug macro outputs

Set `DEBUG_MACRO=1` and enable a feature flag:

- `debug_macro_cli` - prints generated code to stdout during build
- `debug_macro_file` - writes generated code to `target/grand-line/` during build

---

### Design notes

#### What GrandLine does well

1. **Boilerplate close to zero.** `#[model]` alone generates the sea-orm entity, the GraphQL object, the filter input, and the order-by enum; `#[search]`/`#[create]`/`#[update]`/`#[delete]`/`#[detail]`/`#[count]` turn that into full CRUD resolvers. The [simple_todo example](https://github.com/nongdan-dev/grand-line/blob/master/examples/simple_todo/src/main.rs) gets a complete CRUD API in about 100 lines.
2. **Automatic GraphQL look-ahead.** `gql_select` inspects the actual requested fields and only selects those SQL columns - no `SELECT *`, no manual projection code. Most comparable frameworks don't do this.
3. **N+1 handled for `has_one`/`belongs_to` by construction.** These relations batch through a per-request `DataLoader` automatically - nothing to opt into, nothing to hand-write (`has_many`/`many_to_many` don't batch this way, see [limitations](#known-limitations) below).
4. **Transparent per-request transaction.** `GrandLineExtension` opens one transaction lazily on first use and commits/rolls back at the end of the request; resolvers never manage transaction lifecycle themselves.
5. **Authorization is declarative, not `if`/`else` in every resolver.** Column-level policy (field allow/deny) and row-level policy (a runtime script that produces a filter) both attach via the `authz` attribute - e.g. `#[search(Task, authz(realm = "org"))]` - instead of hand-rolled checks scattered through resolver bodies.
6. **Schema wiring generated from source, not maintained by hand.** `grand_line_build` scans your source at compile time and emits the `Query`/`Mutation` `MergedObject` for you - forgetting to register a new resolver isn't a class of bug here.
7. **Fine-grained feature flags.** `model_created_at`, `resolver_tx`, `resolver_authz_row`, and friends let a project opt out of exactly the parts of the convention it doesn't want.

#### Known limitations

- **Macro-generated code is hard to debug by nature, not by neglect.** When something goes wrong inside generated code, the compiler error points into the expansion, not your source. `debug_macro_cli`/`debug_macro_file` (dump the generated code to stdout or `target/grand-line/`) are the standard mitigation available in the Rust macro ecosystem - `sqlx`, `diesel`, and `async-graphql` don't do meaningfully better here, this is a property of how proc-macros report errors, not a gap specific to this framework.
- **One transaction per request, by convention, not by force.** All resolvers in a request share one `DatabaseTransaction` (see [Transactions](#transactions)) - sibling relation resolvers may run as concurrent futures but their SQL still serializes on that one connection. This is a recommendation, not an enforced constraint: the raw `Arc<DatabaseConnection>` is still reachable from `ctx` for any resolver that needs to run something outside the shared transaction (e.g. an operation that must commit independently of the rest of the request).
- **`has_many`/`many_to_many` are not DataLoader-batched.** Only `has_one`/`belongs_to` get the batching treatment described above; a deeply nested search through a to-many relation can still hit N+1.
- **The row-policy DSL is an opaque runtime string.** `RowPolicy` scripts are forwarded verbatim to your own `AuthzHandlers::execute_script` - there's no built-in type checking or test harness for the script format itself, since the format is entirely up to your implementation. Isolate and unit-test `execute_script` directly in your app rather than only exercising it through full GraphQL requests.
- **No subscriptions yet.** `EmptySubscription` is used throughout - there's no real-time/push story today.
- **Missing standard OSS project scaffolding.** No `LICENSE`, CI workflow, `CHANGELOG.md`, or `CONTRIBUTING.md` yet - worth adding before wider public use, independent of the framework's own code quality.

A few things that look like limitations at first read but are intentional, documented trade-offs:

- **`exec_without_ctx` skipping history/`*_by_id` fields is the documented contract, not a bug** - the name says exactly what it does; using it for seeding/batch jobs that don't have a GraphQL context is the intended use case (see [Active model helpers](#active-model-helpers)).
- **No migrations tooling is a deliberate boundary, not an oversight.** Schema migration is a separate concern from GraphQL API generation - bring your own (`sea-orm-cli migrate`, `sqlx migrate`, `refinery`, etc.). Baking one in would couple this crate to a specific migration workflow for no benefit to the GraphQL/resolver layer itself.
- **History recording and delete share one transaction, so there's no create-history-then-fail-delete race** - `History::add` runs inside the same `tx` as the delete it documents; if the delete fails afterward, the whole transaction (history record included) rolls back together.
- **`col_policy`'s `"*"` wildcard always wins over a specific operation entry, by design of an allow-only model.** `ColPolicy` only grants access, it has no deny entry, so there is currently no way to express "`*` allows everything except this one operation." Checking `"*"` first is a direct consequence of that, not an inverted-precedence bug. Revisit if a deny-mode policy entry is ever added (see Roadmap).
- **A row policy entry with no wired-up handler behaves as unrestricted, not as a deny.** `AuthzHandlers::execute_script` defaults to `Ok(None)`, and `authz_row` treats that identically to "no row policy entry for this path" - both mean no filter is applied. This lets a host app introduce `row_policy` entries incrementally without `authz_row` blocking access before `execute_script` is actually implemented for that script. If you need row policies to fail closed while a handler is unimplemented, check for that explicitly in your own `execute_script`.
- **`forgot`'s email-not-found path is a distinguishable 404, so the endpoint currently allows registered-email enumeration.** Most production forgot-password flows return an identical response regardless of whether the email exists, to avoid leaking which emails are registered. Tracked as a to-review tradeoff, not fixed by default, since some deployments intentionally prefer a clear "no account" message over enumeration resistance.

#### Roadmap

Rough priority order for anyone picking this up next:

1. **Migration-adjacent tooling** (optional, separate from core) - a thin wrapper around an existing migration tool, or schema-diff generation from entities, if the project wants an opinionated answer here at all.
2. **`exec_with_by_id(db, Option<String>)`** - a history/audit-aware variant of `exec_without_ctx` for seeding and batch jobs that have a user id but no GraphQL `Context`.
3. **Hook history into raw `Entity::insert_many` call sites**, or document clearly that bulk creates must go through the `Vec<AmWrapper<AmCreate, ..>>` exec path (which already records history per row) rather than calling `insert_many` directly.
4. **Row-policy DSL: dedicated tests and docs** for `AuthzHandlers::execute_script`, with a pattern for unit-testing it independent of full GraphQL requests.
5. **`col_policy` deny-mode entry** - a way to express "allow via `*` except this one operation," so wildcard grants can be narrowed without restructuring the whole policy into explicit per-operation entries.
6. **Subscriptions**, if real-time becomes a requirement - currently the largest capability gap.
7. **Crate publishing readiness** - stabilize the public API surface, add rustdoc to the core traits (`EntityX`, `FilterX`, `OrderBy`, `AuthHandlers`, `AuthzHandlers`, ...), adopt semantic versioning.
8. **Standard OSS scaffolding** - `LICENSE`, CI, `CHANGELOG.md`, `CONTRIBUTING.md`.
