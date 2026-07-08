// Integration tests for row-level authorization via PolicyOperation.row formulas.
//
// Each test creates a schema with a single `rowResult` resolver that calls
// ctx.authz_row::<RowFilter>(). The policy's `row` Rhai script is set per test
// to exercise different injection and evaluation scenarios.
//
// Note: non-client errors (e.g. unknown variable) are masked as InternalServer
// by GrandLineExtension before reaching the GraphQL response.

use grand_line::prelude::*;
use tokio::{runtime::Handle, task::yield_now};

#[path = "../_fixtures/user.rs"]
mod user;
use user::*;

#[path = "../_fixtures/org.rs"]
mod org;
use org::*;

// ---------------------------------------------------------------------------
// Filter type deserialized from the Rhai map result.
// All fields are Option so any partial map from a script can be handled.
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct RowFilter {
    assigned_to_id_eq: Option<String>,
    org_id_eq: Option<String>,
    expires_after: Option<i64>,
}

// ---------------------------------------------------------------------------
// GraphQL output type returned by the test resolver.
// ---------------------------------------------------------------------------

#[derive(SimpleObject)]
struct RowOutput {
    assigned_to_id_eq: Option<String>,
    org_id_eq: Option<String>,
    expires_after: Option<i64>,
}

// ---------------------------------------------------------------------------
// Test resolver: requires org-realm authz, then returns the row filter.
// authz_row returns None when the matched policy operation has no row script.
// ---------------------------------------------------------------------------

#[query(authz(realm = "org"))]
fn row_result() -> Option<RowOutput> {
    ctx.authz().await?;
    let filter: Option<RowFilter> = ctx.authz_row().await?;
    filter.map(|f| RowOutput {
        assigned_to_id_eq: f.assigned_to_id_eq,
        org_id_eq: f.org_id_eq,
        expires_after: f.expires_after,
    })
}

#[derive(Default, MergedObject)]
struct TestQuery(RowResultQuery);

// ---------------------------------------------------------------------------
// Test setup
// ---------------------------------------------------------------------------

struct Setup {
    tmp: TmpDb,
    schema: GraphQLSchema<TestQuery, EmptyMutation, EmptySubscription>,
    user_id: String,
    org_id: String,
}

async fn prepare(row_script: Option<&str>) -> Res<Setup> {
    prepare_with_op_key("rowResult", row_script).await
}

async fn prepare_with_op_key(op_key: &str, row_script: Option<&str>) -> Res<Setup> {
    prepare_inner(op_key, row_script, None).await
}

async fn prepare_with_config(row_script: Option<&str>, cfg: AuthzConfig) -> Res<Setup> {
    prepare_inner("rowResult", row_script, Some(cfg)).await
}

async fn prepare_inner(op_key: &str, row_script: Option<&str>, cfg: Option<AuthzConfig>) -> Res<Setup> {
    let tmp = tmp_db!(User, LoginSession, Org, Role, UserInRole);
    let mut builder = schema_q::<TestQuery>(&tmp.db).data(authz_org_impl::<Org>());
    if let Some(c) = cfg {
        builder = builder.data(c);
    }
    let h = init_common_headers();

    let u = am_create!(User {
        email: "tester@example.com",
        password_hashed: rand_utils::password_hash("pass123")?,
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let ua = Context::get_ua_raw(Context::get_headers_raw(&h))?;
    let secret = rand_utils::secret();
    let ls = am_create!(LoginSession {
        user_id: u.id.clone(),
        secret_hashed: rand_utils::secret_hash(&secret),
        ip: "127.0.0.1",
        ua: ua.to_json()?,
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    let token = rand_utils::qs_token(&ls.id, &secret)?;

    let o = am_create!(Org {
        name: "TestOrg",
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let wc_field = PolicyField {
        allow: true,
        children: Some(hashmap! {
            "**".to_owned() => PolicyField { allow: true, children: None }
        }),
    };
    let ops: PolicyOperations = hashmap! {
        op_key.to_owned() => PolicyOperation {
            inputs: wc_field.clone(),
            output: wc_field.clone(),
            row: row_script.map(|v| v.to_owned()),
        }
    };

    let r = am_create!(Role {
        name: "Tester",
        realm: "org",
        operations: ops.to_json()?,
        org_id: Some(o.id.clone()),
    })
    .exec_without_ctx(&tmp.db)
    .await?;
    am_create!(UserInRole {
        user_id: u.id.clone(),
        role_id: r.id.clone(),
        org_id: Some(o.id.clone()),
    })
    .exec_without_ctx(&tmp.db)
    .await?;

    let mut headers = h;
    headers.append(H_ORG_ID, h_str(&o.id));
    headers.insert(H_AUTHORIZATION, h_bearer(&token));
    let schema = builder.data(headers).finish();

    Ok(Setup {
        tmp,
        schema,
        user_id: u.id,
        org_id: o.id,
    })
}

// GQL query used by every test below.
const Q: &str = "
query {
    rowResult {
        assignedToIdEq
        orgIdEq
        expiresAfter
    }
}
";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// When the policy operation has no row script, authz_row returns None and
// the GQL field is null.
#[tokio::test]
async fn no_row_script_returns_null() -> Res<()> {
    let d = prepare(None).await?;
    let expected = value!({ "rowResult": null });
    exec_assert(&d.schema, Q, None, &expected).await;
    d.tmp.drop().await
}

// current_user is injected into scope as the authenticated user's ID.
#[tokio::test]
async fn current_user_injected() -> Res<()> {
    let d = prepare(Some("#{ assigned_to_id_eq: current_user }")).await?;
    let expected = value!({
        "rowResult": {
            "assignedToIdEq": d.user_id,
            "orgIdEq": null,
            "expiresAfter": null,
        }
    });
    exec_assert(&d.schema, Q, None, &expected).await;
    d.tmp.drop().await
}

// current_org is injected into scope as the org ID from the request header.
#[tokio::test]
async fn current_org_injected() -> Res<()> {
    let d = prepare(Some("#{ org_id_eq: current_org }")).await?;
    let expected = value!({
        "rowResult": {
            "assignedToIdEq": null,
            "orgIdEq": d.org_id,
            "expiresAfter": null,
        }
    });
    exec_assert(&d.schema, Q, None, &expected).await;
    d.tmp.drop().await
}

// Both current_user and current_org are available in the same script.
#[tokio::test]
async fn both_user_and_org_injected() -> Res<()> {
    let d = prepare(Some("#{ assigned_to_id_eq: current_user, org_id_eq: current_org }")).await?;
    let expected = value!({
        "rowResult": {
            "assignedToIdEq": d.user_id,
            "orgIdEq": d.org_id,
            "expiresAfter": null,
        }
    });
    exec_assert(&d.schema, Q, None, &expected).await;
    d.tmp.drop().await
}

// Rhai if/else is an expression and can produce the final map.
#[tokio::test]
async fn conditional_expression_script() -> Res<()> {
    let d = prepare(Some(
        "if true { #{ assigned_to_id_eq: current_user } } else { #{ org_id_eq: current_org } }",
    ))
    .await?;
    let expected = value!({
        "rowResult": {
            "assignedToIdEq": d.user_id,
            "orgIdEq": null,
            "expiresAfter": null,
        }
    });
    exec_assert(&d.schema, Q, None, &expected).await;
    d.tmp.drop().await
}

// let bindings allow computing intermediate values before the final map.
#[tokio::test]
async fn let_binding_before_filter() -> Res<()> {
    let d = prepare(Some("let uid = current_user; #{ assigned_to_id_eq: uid }")).await?;
    let expected = value!({
        "rowResult": {
            "assignedToIdEq": d.user_id,
            "orgIdEq": null,
            "expiresAfter": null,
        }
    });
    exec_assert(&d.schema, Q, None, &expected).await;
    d.tmp.drop().await
}

// `now` is resolved by the built-in NowResolver before eval. The script
// receives it as a UTC millisecond timestamp.
#[tokio::test]
async fn now_resolver_injects_timestamp() -> Res<()> {
    let d = prepare(Some("#{ expires_after: now }")).await?;
    let r = exec_assert_ok(&d.schema, Q, None).await;
    let r = r.data.to_json()?;
    let v = r.pointer("/rowResult/expiresAfter").unwrap_or_default();
    assert!(!v.is_null(), "expiresAfter should not be null, got: {v:?}");
    d.tmp.drop().await
}

// An unknown variable in the row script causes a server-side error.
// GrandLineExtension masks it as InternalServer in the GraphQL response.
#[tokio::test]
async fn unknown_variable_in_script_errors() -> Res<()> {
    let d = prepare(Some("#{ x: unknown_var }")).await?;
    exec_assert_err(&d.schema, Q, None, &CoreGraphQLErr::InternalServer).await;
    d.tmp.drop().await
}

// A Rhai compile error in the row script is also a server-side error.
#[tokio::test]
async fn compile_error_in_script_errors() -> Res<()> {
    let d = prepare(Some("}}{{invalid rhai")).await?;
    exec_assert_err(&d.schema, Q, None, &CoreGraphQLErr::InternalServer).await;
    d.tmp.drop().await
}

// Policy operations with the wildcard key "*" are matched by authz_row
// for any operation name (same lookup logic as the authz directive).
#[tokio::test]
async fn wildcard_operation_key_matched() -> Res<()> {
    // op_key "*" - matches any GQL field name via map.get("*")
    let d = prepare_with_op_key("*", Some("#{ assigned_to_id_eq: current_user }")).await?;
    let expected = value!({
        "rowResult": {
            "assignedToIdEq": d.user_id,
            "orgIdEq": null,
            "expiresAfter": null,
        }
    });
    exec_assert(&d.schema, Q, None, &expected).await;
    d.tmp.drop().await
}

// A call to an unregistered function in the row script causes a Rhai runtime error,
// masked as InternalServer by GrandLineExtension.
#[tokio::test]
async fn unknown_function_in_script_errors() -> Res<()> {
    let d = prepare(Some("#{ x: unknown_fn() }")).await?;
    exec_assert_err(&d.schema, Q, None, &CoreGraphQLErr::InternalServer).await;
    d.tmp.drop().await
}

// Arithmetic on the `now` scope variable produces a computed timestamp.
// This exercises integer expressions in Rhai (now is i64 milliseconds).
#[tokio::test]
async fn arithmetic_on_now_resolver() -> Res<()> {
    let d = prepare(Some("#{ expires_after: now + 86400000 }")).await?;
    let r = exec_assert_ok(&d.schema, Q, None).await;
    let r = r.data.to_json()?;
    let v = r.pointer("/rowResult/expiresAfter").unwrap_or_default();
    assert!(!v.is_null(), "expiresAfter should not be null, got: {v:?}");
    assert!(
        v.as_i64().is_some_and(|v| v > 0),
        "expiresAfter should be a positive integer, got: {v:?}",
    );
    d.tmp.drop().await
}

// intl tagged template literals in row scripts are preprocessed to intl() calls
// before compilation. The built-in 1-arg intl() fn returns the template string as-is.
#[tokio::test]
async fn intl_template_in_row_script() -> Res<()> {
    let d = prepare(Some("#{ assigned_to_id_eq: intl`Hello!` }")).await?;
    let expected = value!({
        "rowResult": {
            "assignedToIdEq": "Hello!",
            "orgIdEq": null,
            "expiresAfter": null,
        }
    });
    exec_assert(&d.schema, Q, None, &expected).await;
    d.tmp.drop().await
}

// A custom FormulaResolver injects named scope variables beyond the built-ins.
// This tests the pre-fetch track: the resolver runs async before Rhai eval.
#[tokio::test]
async fn custom_resolver_injects_value() -> Res<()> {
    struct TierResolver;

    #[async_trait]
    impl FormulaResolver for TierResolver {
        async fn resolve(&self, _name: &str, _ctx: &FormulaCtx<'_>) -> Res<FormulaDynamic> {
            Ok(FormulaDynamic::from("gold".to_owned()))
        }
    }

    let d = prepare_with_config(
        Some("#{ assigned_to_id_eq: user_tier }"),
        AuthzConfig {
            row_graph: FormulaDepGraph::new([
                FormulaDepNode::new("now", [] as [&str; 0], NowResolver),
                FormulaDepNode::new("user_tier", [] as [&str; 0], TierResolver),
            ])?,
            ..Default::default()
        },
    )
    .await?;
    let expected = value!({
        "rowResult": {
            "assignedToIdEq": "gold",
            "orgIdEq": null,
            "expiresAfter": null,
        }
    });
    exec_assert(&d.schema, Q, None, &expected).await;
    d.tmp.drop().await
}

// A custom FormulaResolver can use FormulaCtx to access the request user/org.
// Here the resolver echoes the user_id back under a different variable name.
#[tokio::test]
async fn custom_resolver_reads_ctx_user_id() -> Res<()> {
    struct MirrorUserResolver;

    #[async_trait]
    impl FormulaResolver for MirrorUserResolver {
        async fn resolve(&self, _name: &str, ctx: &FormulaCtx<'_>) -> Res<FormulaDynamic> {
            let id = ctx.user_id.unwrap_or("").to_owned();
            Ok(FormulaDynamic::from(id))
        }
    }

    let d = prepare_with_config(
        Some("#{ org_id_eq: requester_id }"),
        AuthzConfig {
            row_graph: FormulaDepGraph::new([
                FormulaDepNode::new("now", [] as [&str; 0], NowResolver),
                FormulaDepNode::new("requester_id", [] as [&str; 0], MirrorUserResolver),
            ])?,
            ..Default::default()
        },
    )
    .await?;
    let expected = value!({
        "rowResult": {
            "assignedToIdEq": null,
            "orgIdEq": d.user_id,
            "expiresAfter": null,
        }
    });
    exec_assert(&d.schema, Q, None, &expected).await;
    d.tmp.drop().await
}

// When a row script calls db_find_one but no functions are registered,
// Rhai returns "function not found" at runtime (masked as InternalServer).
#[tokio::test]
async fn db_find_one_without_accessor_errors() -> Res<()> {
    let d = prepare(Some(r#"let r = db_find_one("t", #{}); #{ assigned_to_id_eq: r }"#)).await?;
    // Default config has no row_register_fns -> db_find_one is unknown ->
    // Rhai runtime error -> masked as InternalServer by GrandLineExtension.
    exec_assert_err(&d.schema, Q, None, &CoreGraphQLErr::InternalServer).await;
    d.tmp.drop().await
}

// Proves the spawn_blocking fix: a FormulaDbAccessor that calls handle.block_on()
// should NOT panic on current_thread tokio runtime (the default for #[tokio::test]).
// Before the fix (block_in_place), this test would panic.
#[tokio::test]
async fn db_find_one_with_accessor_on_current_thread_runtime() -> Res<()> {
    struct EchoAccessor;

    impl FormulaDbAccessor for EchoAccessor {
        fn find_one_sync(&self, table: &str, _filter: &FormulaMap, handle: &Handle) -> Res<FormulaDynamic> {
            // Use handle.block_on to run async code from this sync context.
            // This is the pattern that block_in_place would panic on for
            // current_thread runtimes, spawn_blocking makes it safe.
            let result = handle.block_on(async move {
                // Simulate a lightweight async operation (no real DB needed).
                yield_now().await;
                table.to_owned()
            });
            Ok(FormulaDynamic::from(result))
        }
        fn find_many_sync(&self, _table: &str, _filter: &FormulaMap, _handle: &Handle) -> Res<Vec<FormulaDynamic>> {
            Ok(vec![])
        }
    }

    let accessor: Arc<dyn FormulaDbAccessor> = Arc::new(EchoAccessor);
    let d = prepare_with_config(
        Some(r#"#{ assigned_to_id_eq: db_find_one("my_table", #{}) }"#),
        AuthzConfig {
            row_register_fns: Some(register_db_fns(accessor)),
            ..Default::default()
        },
    )
    .await?;
    let expected = value!({
        "rowResult": {
            "assignedToIdEq": "my_table",
            "orgIdEq": null,
            "expiresAfter": null,
        }
    });
    exec_assert(&d.schema, Q, None, &expected).await;
    d.tmp.drop().await
}
