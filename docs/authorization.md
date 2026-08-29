# Authorization

The `authz` feature (implies `auth`) provides role-based access control with org scoping, field-level (col) policy checks, and row-level filtering. Like `auth`, it ships primitives, not concrete models - you define your own `Org`, `Role`, and `UserInRole` (any shape you like) and a macro per model derives the DI impls the framework looks them up through. See the [saas example](https://github.com/namnm/grand-line/blob/master/examples/saas/src/authz) for a full implementation.

## Setup

Define your own `Org`, `Role`, and `UserInRole`, and mark each with its macro right under `#[model]`. The macros implement the traits the framework's default DI impls read, so there is no lookup code to write:

```rs
#[model]
#[authz_org]
pub struct Org {
    pub name: String,
}

#[model]
#[authz_role(fallback = "system")]
pub struct Role {
    pub name: String,
    /// Groups multiple roles into a realm, e.g. "org" or "system".
    pub realm: String,
    pub col_policy: JsonValue,
    pub row_policy: JsonValue,
    /// None for realm-wide roles not tied to a single org.
    pub org_id: Option<String>,
}

#[model]
#[authz_user_in_role]
pub struct UserInRole {
    pub user_id: String,
    pub role_id: String,
    pub org_id: Option<String>,
}
```

| Macro                   | Fields it reads, on top of `id` from `#[model]` |
| ----------------------- | ----------------------------------------------- |
| `#[authz_org]`          | none, it only marks the org lookup target       |
| `#[authz_role]`         | `realm`, `col_policy`, `row_policy`, `org_id`   |
| `#[authz_user_in_role]` | `user_id`, `role_id`, `org_id`                  |

`#[authz_role]` and `#[authz_user_in_role]` also mark their model org scoped, same as [`#[authz_org_id]`](#org-scoped-models) below.

`fallback = "system"` is optional: when no role matches the requested realm, the lookup retries in that realm with no org scoping, i.e. a `system` role acts on every org. Leave it off to never fall back.

Wire the derived impls onto the schema:

```rs
GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
    .extension(GrandLineExtension)
    .data(Arc::new(db.clone()))
    .data(Org::authz_default_impl())
    .data(Role::authz_default_impl::<UserInRole>())
    .data(session_impl) // still needed - authz is built on top of auth
    .data(otp_impl)
    .finish()
```

`AuthzOrgImpl`/`AuthzRoleImpl` are mandatory, same as `AuthSessionImpl`/`AuthOtpImpl` - missing either errors with `OrgImplNotFound`/`RoleImplNotFound`. `AuthzConfig` is optional (falls back to `AuthzConfig::default()`), and lets you override the header names, swap `unauthorized_err` (e.g. for `CoreDbErr::Db404` if you don't want to reveal that a resource exists), or plug in `AuthzHandlers` for row policy (see [below](#row-policy-and-authz_row)).

Every `authz`-gated request must include the `X-Role-Id` header (the id of the role being acted as), plus `X-Org-Id` unless the check opts out via `skip_org()`.

### Writing the role lookup by hand

The macros are a shortcut, not a requirement. When your role matching needs something the default doesn't express (roles resolved through a group table, more than one fallback realm), drop `#[authz_role]` and implement `AuthzRoleImpl` yourself - given the request's realm/org/user requirements plus the `X-Role-Id` header value, find the matching role (or `None`):

```rs
pub struct MyRoleImpl;
#[async_trait]
impl AuthzRoleImpl for MyRoleImpl {
    async fn find_matching(
        &self,
        check: &AuthzEnsure, // { realm, org: bool, user: bool } - what this resolver's guard requires
        role_id: &str,       // from the X-Role-Id header
        org_id: Option<&str>, // from the X-Org-Id header, only Some if check.org
        user_id: Option<&str>, // from ctx.auth(), only Some if check.user
        db: &ConnX<'_>,
    ) -> Res<Option<AuthzRoleMatch>> {
        let Some(role) = Role::find()
            .include_deleted(false)
            .filter_by_id(role_id)
            .filter(RoleColumn::Realm.eq(&check.realm))
            .one(db)
            .await?
        else {
            return Ok(None);
        };
        // also check UserInRole for (role_id, org_id, user_id) if your app requires it
        Ok(Some(AuthzRoleMatch {
            role_id: role.id,
            col_policy: ColPolicy::from_json(role.col_policy)?,
            row_policy: RowPolicy::from_json(role.row_policy)?,
        }))
    }
}
```

```rs
let role_impl: Box<dyn AuthzRoleImpl> = Box::new(MyRoleImpl);
```

## `authz_ensure` guard

`authz` ships one `ctx` guard, `authz_ensure(AuthzEnsure)`, used through the [`check`](resolvers.md#guards-check) resolver attribute. Realm categorizes scope - a common convention, not a fixed enum (the realm string is whatever you compare against in your own `AuthzRoleImpl`):

| realm    | `AuthzEnsure`                                         | Checks     |
| -------- | ----------------------------------------------------- | ---------- |
| `org`    | `AuthzEnsure::realm("org")`                           | user + org |
| `system` | `AuthzEnsure::realm("system").skip_org()`             | user only  |
| `public` | `AuthzEnsure::realm("public").skip_org().skip_user()` | none       |

Wrap the realms your app uses in one guard each and put the trait in your prelude, so a resolver only has to name the realm:

```rs
#[async_trait]
pub trait MyCheck<'a>
where
    Self: AuthzEnsureContext<'a>,
{
    /// Requires an org realm role scoped to the request org and user.
    async fn authz_org(&self) -> Res<()> {
        self.authz_ensure(AuthzEnsure::realm("org")).await
    }
    /// Requires a system realm role, not scoped to any org.
    async fn authz_system(&self) -> Res<()> {
        self.authz_ensure(AuthzEnsure::realm("system").skip_org()).await
    }
}

#[async_trait]
impl<'a> MyCheck<'a> for Context<'a> {
}
```

```rs
// Org-scoped: requires Authorization + X-Org-Id + X-Role-Id headers
#[query(check = authz_org)]
fn org_dashboard() -> OrgGql {
    let org_id = ctx.authz().await?;
    Org::find_by_id(&org_id).gql_select(ctx)?.one_or_404(db).await?
}

// System-wide: requires Authorization + X-Role-Id (no X-Org-Id)
#[query(check = authz_system)]
fn system_dashboard() -> String {
    "ok".to_string()
}

// Works on all CRUD macros - use authz_row for row-level filtering
#[search(Task, check = authz_org)]
fn resolver() {
    ctx.authz_row::<TaskFilter>().await?.into()
}
```

By default a mismatch (missing role, wrong org, wrong realm, user not assigned) surfaces as `AuthzErr::Unauthorized`; set `AuthzConfig.unauthorized_err` to change it.

The guard is only meaningful on a root resolver: its result is cached under the root field and every nested relation reads that cache. Reading `ctx.authz()`/`ctx.authz_row()` in a resolver no guard ran on errors with `AuthzErr::MissingGuard`.

## `ctx` methods

```rs
ctx.authz().await?          // -> Res<String>, the verified org_id from X-Org-Id
ctx.authz_role().await?     // -> Res<Arc<AuthzCacheItem>>, the matched role's cached col_policy/row_policy
ctx.authz_row::<F>().await? // -> Res<Option<F>>, row-level filter from the role's row_policy script
ctx.org_unchecked().await?  // -> Res<Arc<OrgMinimal>>, org from X-Org-Id without an auth/authz check
```

### Org scoped models

Mark any model that belongs to a single org with `#[authz_org_id]` (already implied by `#[authz_role]`/`#[authz_user_in_role]`), and four extra helpers cover the common "CRUD scoped to the current authz org" boilerplate:

```rs
ctx.authz_org_search::<Role>().await?         // -> Res<Search<RoleOrderBy>>, filtered to the current org
ctx.authz_org_filter::<Role>().await?         // -> Res<Filter>, same filter for count/detail
ctx.authz_org_one_or_404::<Role>(&id).await?  // -> Res<RoleSql>, fetch by id scoped to the current org
ctx.authz_org_soft_delete::<Role>(&id).await? // -> Res<RoleGql>, soft-delete by id scoped to the current org
```

```rs
#[model]
#[authz_org_id]
pub struct Task {
    pub title: String,
    pub org_id: String,
}

#[search(Role, check = authz_org)]
fn resolver() {
    ctx.authz_org_search::<Role>().await?
}

#[detail(Role, check = authz_org)]
fn resolver() {
    ctx.authz_org_filter::<Role>().await?
}

#[mutation(check = authz_org)]
fn role_delete(id: String) -> RoleGql {
    ctx.authz_org_soft_delete::<Role>(&id).await?
}
```

## Col policy structure

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

## Row policy and `authz_row`

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

Inside a resolver, call `ctx.authz_row::<F>()`. Authorization is already guaranteed by the guard. Returns `None` when no entry exists for this field (all rows accessible), or `Some(F)` when the script produced a filter:

```rs
#[search(Task, check = authz_org)]
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
