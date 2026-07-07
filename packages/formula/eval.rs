use crate::cache::{map_rhai_pos, parse_and_cache};
use crate::dep_graph::FormulaDepGraph;
use crate::engine::{BUILTIN_SCOPE_VARS, make_base_engine};
use crate::err::FormulaErr;
use crate::opts::FormulaOptions;
use crate::resolver::FormulaCtx;
use _core::prelude::*;
use rhai::{Dynamic, Engine, Scope};
use tokio::task::spawn_blocking;

/// Compile, validate, resolve, and evaluate a formula script.
///
/// Returns the script's result as a `JsonValue`.
///
/// `register_fns` is called once per eval to register functions on a fresh
/// Rhai engine. Pass an empty closure `|_| {}` when no functions are needed.
/// Use `register_db_fns` to add `db_find_one` / `db_find_many` support.
///
/// Variables are supplied via `graph` (async dep-graph resolvers) and the
/// built-in scope (`current_user`, `current_org`). The `locale` value is
/// forwarded to resolvers via `FormulaCtx`.
pub async fn eval_formula(
    script: &str,
    user_id: Option<&str>,
    org_id: Option<&str>,
    locale: &str,
    tx: &DatabaseTransaction,
    graph: &FormulaDepGraph,
    opts: &FormulaOptions,
    register_fns: impl FnOnce(&mut Engine) + Send,
) -> Res<JsonValue> {
    let script_deps = parse_and_cache(script)?;

    // Validate variable references (skip locally-defined `let` variables).
    for name in script_deps.var_deps.iter() {
        if script_deps.local_vars.contains(name) {
            continue;
        }
        if !BUILTIN_SCOPE_VARS.contains(&name.as_str()) && !graph.contains(name) {
            return Err(FormulaErr::UnknownVar(name.clone()).into());
        }
    }

    // Pre-fetch track: resolve all graph nodes in topological order.
    // Each resolver sees previously resolved values in ctx.resolved.
    let mut resolved: HashMap<String, Dynamic> = HashMap::new();
    for name in graph.topo_order() {
        let node = graph
            .get_node(name)
            .ok_or_else(|| FormulaErr::UnknownVar(name.clone()))?;
        let val = {
            let ctx = FormulaCtx {
                user_id,
                org_id,
                locale,
                tx,
                resolved: &resolved,
            };
            node.resolver.resolve(name, &ctx).await?
        };
        resolved.insert(name.to_owned(), val);
    }

    let mut scope = Scope::new();
    if let Some(id) = user_id {
        scope.push_constant("current_user", id.to_owned());
    }
    if let Some(id) = org_id {
        scope.push_constant("current_org", id.to_owned());
    }
    for (name, val) in resolved {
        scope.push_constant(name, val);
    }

    // Build per-eval engine: fresh base with caller limits + registered functions.
    let mut engine = make_base_engine(opts);
    register_fns(&mut engine);

    let ast = Arc::clone(&script_deps.ast);

    // Rhai eval runs on a spawn_blocking thread so that FormulaDbAccessor
    // implementations can use handle.block_on(async { ... }) without
    // deadlocking. spawn_blocking creates a dedicated OS thread outside the
    // tokio worker pool -- handle.block_on() works on both current_thread and
    // multi_thread runtimes.
    let eval_result = spawn_blocking(move || engine.eval_ast_with_scope::<Dynamic>(&mut scope, &ast))
        .await
        .map_err(|e| FormulaErr::Eval(format!("eval thread panicked: {e}")))?;

    let dynamic = eval_result.map_err(|e| {
        let hint = script_deps
            .source_map
            .as_ref()
            .and_then(|sm| map_rhai_pos(sm, e.position()))
            .unwrap_or_default();
        FormulaErr::Eval(format!("{e}{hint}"))
    })?;

    let json = dynamic.to_json().map_err(|e| FormulaErr::Serialize(e.to_string()))?;
    Ok(json)
}
