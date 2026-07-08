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
- [Schema collector](#schema-collector)
- [Resolver bodies](#resolver-bodies)
- [Context](#context)
- [Transactions](#transactions)
- [Relationships](#relationships)
- [Filtering and sorting](#filtering-and-sorting)
- [Active model helpers](#active-model-helpers)
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
  - [Policy structure](#policy-structure)
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
    (None, None)
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

Opt-out per model:

```rs
#[model(created_at=false)]   // no created_at / created_by_id
#[model(updated_at=false)]   // no updated_at / updated_by_id
#[model(deleted_at=false)]   // no deleted_at / deleted_by_id - also disables soft-delete
#[model(by_id=false)]        // no *_by_id fields
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

**`#[graphql(skip)]`** - stored in DB, hidden from the GraphQL schema:

```rs
#[graphql(skip)]
pub password_hashed: String,
```

**`#[sql_expr(...)]`** - GraphQL-only computed column, evaluated by the database:

```rs
#[sql_expr(Expr::col(Column::Price).mul(Expr::val(1.0).sub(Expr::col(Column::DiscountPercentage).div(100.0))))]
pub discounted_price: f64,
```

**`#[resolver(sql_dep = "col1, col2")]`** - GraphQL-only field resolved in Rust. Requires a `resolve_{field_name}` function in the same scope:

```rs
#[resolver(sql_dep = "first_name, last_name")]
pub full_name: String,

async fn resolve_full_name(u: &UserGql, _: &Context<'_>) -> Res<String> {
    Ok(format!("{} {}", u.first_name.ok_or(CoreDbErr::GqlResolverNone)?,
                        u.last_name.ok_or(CoreDbErr::GqlResolverNone)?))
}
```

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

When the function is named `resolver`, the GraphQL field defaults to `{Model}{Operation}` (e.g. `todoSearch`). Any other name overrides it.

The input type for `#[create]` / `#[update]` is the PascalCase of the GraphQL field name.

| Macro       | Body returns                    | Injected locals                                 | Output              |
| ----------- | ------------------------------- | ----------------------------------------------- | ------------------- |
| `#[search]` | `(extra_filter, default_order)` | `filter`, `order_by`, `page`, `include_deleted` | `Vec<TodoGql>`      |
| `#[count]`  | `extra_filter`                  | `filter`, `include_deleted`                     | `u64`               |
| `#[detail]` | nothing (pre-fetch hook)        | `id`, `include_deleted`                         | `Option<TodoGql>`   |
| `#[create]` | `TodoActiveModel`               | `data: TodoCreate`                              | `TodoGql`           |
| `#[update]` | `TodoActiveModel`               | `id`, `data: TodoUpdate`                        | `TodoGql`           |
| `#[delete]` | nothing (pre-delete hook)       | `id`, `permanent: Option<bool>`                 | `TodoGql` (id only) |

```rs
#[search(Todo)]
fn resolver() {
    let extra = filter!(Todo {
        content_starts_with: "2024",
    });
    (Some(extra), Some(order_by!(Todo[CreatedAtDesc])))
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
fn resolver() {}
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
        .filter(f.clone().into_condition())
        .exec(tx)
        .await?;
    f.gql_select_id().all(tx).await?
}
```

These generate `TodoCountDoneQuery` / `TodoDeleteDoneMutation` structs for use in `MergedObject`.

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
fn main() {
    grand_line_build::generate_schema();
}
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
    // ok
    if missing {
        return Err(MyErr::NotFound.into());
    }

    // this will not work
    // if present {
    //     return "ok".to_string();
    // }

    // ok
    "ok".to_string()
}
```

Use `resolver_inputs` to define fully custom parameters:

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
| `ctx.auth_with_cache().await?`               | `Arc<Option<LoginSessionSql>>` | Current session or `None`                                        |
| `ctx.auth_ensure_authenticated().await?`     | `()`                           | Errors if no session                                             |
| `ctx.auth_ensure_not_authenticated().await?` | `()`                           | Errors if already logged in                                      |

**Authz (`grand_line_authz`)**

| Method                          | Returns           | Description                              |
| ------------------------------- | ----------------- | ---------------------------------------- |
| `ctx.authz().await?`            | `String`          | Verified `org_id` from `X-Org-Id` header |
| `ctx.authz_role().await?`       | `RoleSql`         | The matched `Role` row                   |
| `ctx.org_unauthorized().await?` | `Arc<OrgMinimal>` | Org from `X-Org-Id` without auth check   |

---

### Transactions

`GrandLineExtension` manages one lazy transaction per request - commits on success, rolls back on any error.

```rs
GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
    .extension(GrandLineExtension)
    .data(Arc::new(db.clone()))
    .finish()
```

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

Soft-deleted related records are excluded by default. Override per field:

```graphql
query {
    userDetail(id: "...") {
        profile(includeDeleted: true) { bio }
        orgs(filter: { deletedAt_ne: null }) { name }
    }
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

### Active model helpers

```rs
// auto id, created_at, updated_at
am_create!(Todo {
    content: "hello",
    done: false,
})
// auto updated_at
am_update!(Todo {
    id: id.clone(),
    content: "new",
})
// auto deleted_at, updated_at
am_soft_delete!(Todo {
    id: id.clone(),
})
// auto *_by_id
am.exec(ctx)
// without *_by_id
am.exec_without_ctx(tx)
// into sea orm active model
am.into_active_model()

Todo::soft_delete_by_id(&id)?.exec(ctx).await?;
Todo::soft_delete_many()?.filter(condition).exec(ctx).await?;

let todo: TodoSql = Todo::find_by_id(&id).one_or_404(tx).await?;
Todo::find_by_id(&id).exists_or_404(tx).await?;
filter.gql_select_id().all(tx).await?
```

---

### Error handling

```rs
#[grand_line_err]
enum MyErr {
    #[error("record not found")]
    // forwarded to response as-is
    #[client]
    NotFound,

    // client sees generic "internal server error"
    #[error("oops")]
    InternalProblem,
}

// Raise from any resolver:
Err(MyErr::NotFound)?;

// Downcast from response error:
error.source
    .as_deref()
    .and_then(|e| e.downcast_ref::<GrandLineErr>())
    // e.g. "NotFound"
    .map(|e| e.0.code());
```

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
    .data(AuthUserImpl::<User>::default())
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
    fn password_col() -> UserColumn {
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
        secret  # save this - needed in step 2
    }
}

# Step 2
mutation {
    registerResolve(data: { id: "...", secret: "...", otp: "123456" }) {
        secret  # Bearer token for subsequent requests
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
fn my_profile() -> UserGql { ... }

#[mutation(auth(unauthenticated))]
fn register() -> AuthOtpWithSecret { ... }

#[search(Todo, auth)]
fn resolver() { (None, None) }
```

#### Customizing behavior

`AuthHandlers` hooks into OTP and password logic (not user-model specific):

```rs
struct MyHandlers;

#[async_trait]
impl AuthHandlers for MyHandlers {
    async fn otp(&self, _: &Context<'_>) -> Res<String> {
        // custom OTP generator
        Ok(generate_otp())
    }
    async fn on_otp_create(&self, _: &Context<'_>, otp: &AuthOtpSql, raw: &str) -> Res<()> {
        // send the OTP by email
        send_email(&otp.email, raw).await
    }
}

AuthConfig { handlers: Arc::new(MyHandlers), ..Default::default() }
```

`AuthUserHandlers<U>` hooks into the user lifecycle. The callbacks receive your full `U::M` model:

```rs
struct MyUserHandlers;

#[async_trait]
impl AuthUserHandlers<User> for MyUserHandlers {
    async fn on_register_resolve(&self, ctx: &Context<'_>, user: &UserSql, _: &LoginSessionSql) -> Res<()> {
        let tx = &*ctx.tx().await?;
        // user.id, user.display_name, etc. are all available
        am_create!(UserProfile {
            user_id: user.id.clone(),
            bio: "",
        }).exec(ctx).await?;
        Ok(())
    }
}

AuthUserImpl::<User> { handlers: Arc::new(MyUserHandlers) }
```

---

### Authorization

`grand_line_authz` provides role-based access control with org scoping and field-level policy checks.

#### Setup

Define your `Org` model (see [next section](#defining-your-org-model)), then wire up the schema:

```rs
GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
    .extension(GrandLineExtension)
    .data(Arc::new(db.clone()))
    .data(AuthConfig::default())
    .data(AuthUserImpl::<User>::default())
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

```rs
am_create!(Role {
    name: "Org Admin", realm: "org",
    org_id: Some(org_id.clone()),
    operations: operations.to_json()?,
})
.exec(ctx).await?;

am_create!(UserInRole {
    user_id: user_id.clone(), role_id: role_id.clone(),
    // must match role's org_id
    org_id: Some(org_id.clone()),
})
.exec(ctx).await?;
```

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
// Org-scoped: requires Authorization + X-Org-Id headers
#[query(authz(realm = "org"))]
fn org_dashboard() -> OrgGql {
    let org_id = ctx.authz().await?;
    Org::find_by_id(&org_id).gql_select(ctx)?.one_or_404(tx).await?
}

// System-wide: requires Authorization only
#[query(authz(realm = "system", skip_org))]
fn system_dashboard() -> String {
    "ok".to_string()
}

// Works on all CRUD macros
#[search(Todo, authz(realm = "org"))]
fn resolver() {
    (None, None)
}
```

Use `ctx.authz_role().await?` to get the matched `Role` row inside any authz-guarded resolver.

#### Policy structure

Each `Role.operations` is a JSON-encoded `PolicyOperations` map that controls allowed GraphQL inputs and output fields:

```rs
pub type PolicyOperations = HashMap<String, PolicyOperation>;

pub struct PolicyOperation {
    pub inputs: PolicyField, // allowed GraphQL arguments
    pub output: PolicyField, // allowed response fields
}

pub struct PolicyField {
    pub allow: bool,
    pub children: Option<PolicyFields>, // HashMap<String, PolicyField>
}
```

Key is the GraphQL operation name, or `"*"` for all. Wildcards in children:

| Key    | Meaning                            |
| ------ | ---------------------------------- |
| `"*"`  | Allow any direct child field       |
| `"**"` | Allow any nested field recursively |

**Allow everything:**

```rs
let all = PolicyField { allow: true, children: Some(hashmap! {
    "**".to_owned() => PolicyField { allow: true, children: None },
}) };
let ops: PolicyOperations = hashmap! {
    "*".to_owned() => PolicyOperation { inputs: all.clone(), output: all },
};
```

**Restrict to specific fields:**

```rs
let ops: PolicyOperations = hashmap! {
    "todoSearch".to_owned() => PolicyOperation {
        inputs: PolicyField { allow: true, children: Some(hashmap! {
            "filter".to_owned() => PolicyField { allow: true, children: Some(hashmap! {
                "**".to_owned() => PolicyField { allow: true, children: None },
            }) },
        }) },
        output: PolicyField { allow: true, children: Some(hashmap! {
            "id".to_owned()      => PolicyField { allow: true, children: None },
            "content".to_owned() => PolicyField { allow: true, children: None },
        }) },
    },
};
```

---

### Debug macro outputs

Set `DEBUG_MACRO=1` and enable a feature flag:

- `debug_macro_cli` - prints generated code to stdout during build
- `debug_macro_file` - writes generated code to `target/grand-line/` during build
