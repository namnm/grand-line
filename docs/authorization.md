# Authorization

The `authz` feature (implies `auth`) provides role-based access control with org scoping, field-level (col) policy checks, and row-level filtering. Like `auth`, it ships primitives, not concrete models - you define your own `Org`, `Role`, and `UserInRole` (any shape you like) and implement two DI traits so the framework can look them up. See the [saas example](https://github.com/nongdan-dev/grand-line/blob/master/examples/saas/src/authz) for a full implementation.

## Setup

Define `Org` and implement the marker trait `AuthzOrg` on it - this gets you a default `AuthzOrgImpl` for free:

```rs
```

Define `Role` and `UserInRole` yourself, then implement `AuthzRoleImpl` - given the current request's realm/org/user requirements plus the `X-Role-Id` header value, find the matching role (or `None`):

```rs
pub struct MyRoleImpl;
#[async_trait]
impl AuthzRoleImpl for MyRoleImpl {
    async fn find_matching(
        &self,
        check: &AuthzEnsure, // { realm, org: bool, user: bool } - what this resolver's #[authz] requires
        role_id: &str,       // from the X-Role-Id header
        org_id: Option<&str>, // from the X-Org-Id header, only Some if check.org
        user_id: Option<&str>, // from ctx.auth(), only Some if check.user
        tx: &DatabaseTransaction,
    ) -> Res<Option<AuthzRoleMatch>> {
        let Some(role) = Role::find()
            .include_deleted(false)
            .filter_by_id(role_id)
            .filter(RoleColumn::Realm.eq(&check.realm))
            .one(tx)
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

Wire both onto the schema:

```rs
let org_impl = Org::authz_default_impl();
let role_impl: Box<dyn AuthzRoleImpl> = Box::new(MyRoleImpl);

GraphQLSchema::build(Query::default(), Mutation::default(), EmptySubscription)
    .extension(GrandLineExtension)
    .data(Arc::new(db.clone()))
    .data(org_impl)
    .data(role_impl)
    .data(session_impl) // still needed - authz is built on top of auth
    .data(otp_impl)
    .finish()
```

`AuthzOrgImpl`/`AuthzRoleImpl` are mandatory, same as `AuthSessionImpl`/`AuthOtpImpl` - missing either errors with `OrgImplNotFound`/`RoleImplNotFound`. `AuthzConfig` is optional (falls back to `AuthzConfig::default()`), and lets you override the header names, swap `unauthorized_err` (e.g. for `CoreDbErr::Db404` if you don't want to reveal that a resource exists), or plug in `AuthzHandlers` for row policy (see [below](#row-policy-and-authz_row)).

Every `authz`-gated request must include the `X-Role-Id` header (the id of the role being acted as), plus `X-Org-Id` unless the resolver's realm opts out via `skip_org`.

## `authz` attribute

Realm categorizes scope - a common convention, not a fixed enum (the realm string is whatever you compare against in your own `AuthzRoleImpl`):

| realm    | Attribute                                         | Checks     |
| -------- | ------------------------------------------------- | ---------- |
| `org`    | `#[authz(realm = "org")]`                         | user + org |
| `system` | `#[authz(realm = "system", skip_org)]`            | user only  |
| `public` | `#[authz(realm = "public", skip_user, skip_org)]` | none       |

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

By default a mismatch (missing role, wrong org, wrong realm, user not assigned) surfaces as `AuthzErr::Unauthorized`; set `AuthzConfig.unauthorized_err` to change it.

## `ctx` methods

```rs
ctx.authz().await?          // -> Res<String>, the verified org_id from X-Org-Id
ctx.authz_role().await?     // -> Res<Arc<AuthzCacheItem>>, the matched role's cached col_policy/row_policy
ctx.authz_row::<F>().await? // -> Res<Option<F>>, row-level filter from the role's row_policy script
ctx.org_unchecked().await?  // -> Res<Arc<OrgMinimal>>, org from X-Org-Id without an auth/authz check
```

For a model implementing `AuthzImplOrgId` (`fn col_org_id() -> Self::C`, one method, mark any org-scoped model with it), four extra helpers cover the common "CRUD scoped to the current authz org" boilerplate:

```rs
ctx.authz_org_search::<Role>().await?         // -> Res<Search<RoleOrderBy>>, filtered to the current org
ctx.authz_org_filter::<Role>().await?         // -> Res<Filter>, same filter for count/detail
ctx.authz_org_one_or_404::<Role>(&id).await?  // -> Res<RoleSql>, fetch by id scoped to the current org
ctx.authz_org_soft_delete::<Role>(&id).await? // -> Res<RoleGql>, soft-delete by id scoped to the current org
```

```rs
#[model]
pub struct Role {
    pub name: String,
    pub realm: String,
    pub col_policy: JsonValue,
    pub row_policy: JsonValue,
    pub org_id: Option<String>,
}

impl AuthzImplOrgId for Role {
    fn col_org_id() -> Self::C {
        RoleColumn::OrgId
    }
}

#[search(Role, authz(realm = "org"))]
fn resolver() {
    ctx.authz_org_search::<Role>().await?
}

#[detail(Role, authz(realm = "org"))]
fn resolver() {
    ctx.authz_org_filter::<Role>().await?
}

#[mutation(authz(realm = "org"))]
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
