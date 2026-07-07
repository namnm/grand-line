pub use grand_line::prelude::*;

// ---------------------------------------------------------------------------
// Test resolver -- returns a fixed value, optionally read from ctx.resolved
// ---------------------------------------------------------------------------

struct FixedResolver(i64);
#[async_trait]
impl FormulaResolver for FixedResolver {
    async fn resolve(&self, _name: &str, _ctx: &FormulaCtx<'_>) -> Res<FormulaDynamic> {
        Ok(FormulaDynamic::from(self.0))
    }
}

struct DependentResolver;
#[async_trait]
impl FormulaResolver for DependentResolver {
    async fn resolve(&self, _name: &str, ctx: &FormulaCtx<'_>) -> Res<FormulaDynamic> {
        let base = ctx.resolved.get("base").and_then(|d| d.as_int().ok()).unwrap_or(0);
        Ok(FormulaDynamic::from(base + 1))
    }
}

async fn tx(db: &DatabaseConnection) -> Res<DatabaseTransaction> {
    let t = db.begin().await?;
    Ok(t)
}

// ---------------------------------------------------------------------------
// Basic evaluation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn eval_basic_arithmetic() -> Res<()> {
    let tmp = tmp_db!();
    let t = tx(&tmp.db).await?;

    let graph = FormulaDepGraph::empty();
    let opts = FormulaOptions::default();
    let r = eval_formula("1 + 1", None, None, "en", &t, &graph, &opts, |_| {}).await?;

    pretty_eq!(r, JsonValue::from(2), "1 + 1 should evaluate to 2");

    tmp.drop().await
}

#[tokio::test]
async fn eval_uses_current_user_and_org() -> Res<()> {
    let tmp = tmp_db!();
    let t = tx(&tmp.db).await?;

    let graph = FormulaDepGraph::empty();
    let opts = FormulaOptions::default();
    let r = eval_formula(
        "current_user + \"@\" + current_org",
        Some("olivia"),
        Some("fringe_division"),
        "en",
        &t,
        &graph,
        &opts,
        |_| {},
    )
    .await?;

    pretty_eq!(
        r,
        JsonValue::from("olivia@fringe_division"),
        "current_user and current_org should be available as builtin scope vars",
    );

    tmp.drop().await
}

// ---------------------------------------------------------------------------
// Unknown variable validation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn eval_unknown_var_returns_err() -> Res<()> {
    let tmp = tmp_db!();
    let t = tx(&tmp.db).await?;

    let graph = FormulaDepGraph::empty();
    let opts = FormulaOptions::default();
    let r = eval_formula(
        "walter_bishop_secret_formula",
        None,
        None,
        "en",
        &t,
        &graph,
        &opts,
        |_| {},
    )
    .await;

    pretty_eq!(
        r.is_err(),
        true,
        "referencing a var with no matching graph node or builtin should return an error",
    );

    tmp.drop().await
}

#[tokio::test]
async fn eval_let_bound_var_is_not_unknown() -> Res<()> {
    let tmp = tmp_db!();
    let t = tx(&tmp.db).await?;

    let graph = FormulaDepGraph::empty();
    let opts = FormulaOptions::default();
    let r = eval_formula("let x = 41; x + 1", None, None, "en", &t, &graph, &opts, |_| {}).await?;

    pretty_eq!(
        r,
        JsonValue::from(42),
        "a let-bound local var should not require a graph node"
    );

    tmp.drop().await
}

// ---------------------------------------------------------------------------
// try/catch: caught error variable must not be treated as an unknown var
// ---------------------------------------------------------------------------

#[tokio::test]
async fn eval_try_catch_binds_caught_error_locally() -> Res<()> {
    let tmp = tmp_db!();
    let t = tx(&tmp.db).await?;

    let graph = FormulaDepGraph::empty();
    let opts = FormulaOptions::default();
    let script = r#"try { throw "the pattern"; } catch (err) { err }"#;
    let r = eval_formula(script, None, None, "en", &t, &graph, &opts, |_| {}).await?;

    pretty_eq!(
        r,
        JsonValue::from("the pattern"),
        "the catch variable should be locally bound, not flagged as an unknown external var",
    );

    tmp.drop().await
}

// ---------------------------------------------------------------------------
// Dependency graph resolution
// ---------------------------------------------------------------------------

#[tokio::test]
async fn eval_default_graph_exposes_now() -> Res<()> {
    let tmp = tmp_db!();
    let t = tx(&tmp.db).await?;

    let graph = FormulaDepGraph::default();
    let opts = FormulaOptions::default();
    let r = eval_formula("now > 0", None, None, "en", &t, &graph, &opts, |_| {}).await?;

    pretty_eq!(
        r,
        JsonValue::from(true),
        "the default graph's now node should resolve to a positive timestamp"
    );

    tmp.drop().await
}

#[tokio::test]
async fn eval_resolver_sees_earlier_resolved_values() -> Res<()> {
    let tmp = tmp_db!();
    let t = tx(&tmp.db).await?;

    let base = FormulaDepNode::new("base", [] as [&str; 0], FixedResolver(41));
    let derived = FormulaDepNode::new("derived", ["base"], DependentResolver);
    let graph = FormulaDepGraph::new([base, derived])?;
    let opts = FormulaOptions::default();
    let r = eval_formula("derived", None, None, "en", &t, &graph, &opts, |_| {}).await?;

    pretty_eq!(
        r,
        JsonValue::from(42),
        "derived's resolver should see base's already-resolved value"
    );

    tmp.drop().await
}

#[test]
fn dep_graph_cyclic_dependency_returns_err() {
    let a = FormulaDepNode::new("a", ["b"], FixedResolver(1));
    let b = FormulaDepNode::new("b", ["a"], FixedResolver(2));
    let r = FormulaDepGraph::new([a, b]);

    pretty_eq!(
        r.is_err(),
        true,
        "a cycle between two nodes should be rejected at construction"
    );
}

#[test]
fn dep_graph_unknown_dependency_returns_err() {
    let a = FormulaDepNode::new("a", ["missing"], FixedResolver(1));
    let r = FormulaDepGraph::new([a]);

    pretty_eq!(
        r.is_err(),
        true,
        "a dep name with no matching node should be rejected at construction"
    );
}

#[test]
fn dep_graph_topo_order_puts_deps_before_dependents() {
    let a = FormulaDepNode::new("a", [] as [&str; 0], FixedResolver(1));
    let b = FormulaDepNode::new("b", ["a"], FixedResolver(2));
    let c = FormulaDepNode::new("c", ["b"], FixedResolver(3));
    // Construct out of dependency order to prove topo_sort, not insertion order, wins.
    let graph = FormulaDepGraph::new([c, a, b]).unwrap_or_else(|_| FormulaDepGraph::empty());

    let order = graph.topo_order();
    let pos_a = order.iter().position(|n| n == "a").unwrap_or(usize::MAX);
    let pos_b = order.iter().position(|n| n == "b").unwrap_or(usize::MAX);
    let pos_c = order.iter().position(|n| n == "c").unwrap_or(usize::MAX);

    pretty_eq!(pos_a < pos_b, true, "a should come before b in topo order");
    pretty_eq!(pos_b < pos_c, true, "b should come before c in topo order");
}
