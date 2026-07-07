use crate::prelude::*;
use rhai::{AST, ASTNode, Expr, Position, Stmt};
use std::sync::RwLock;

/// Parsed and analyzed form of a formula script: its compiled AST plus the
/// external variable names it references and the local names it binds itself.
pub struct ScriptDeps {
    pub ast: Arc<AST>,
    /// Variables referenced in the script (Expr::Variable nodes).
    pub var_deps: Arc<HashSet<String>>,
    /// Variables declared with let or const (Stmt::Var nodes). Excluded from
    /// validation -- locally defined, not external scope requirements.
    pub local_vars: Arc<HashSet<String>>,
    /// Source map from preprocessed script positions back to original positions.
    /// Present only when preprocess_intl_template_with_map transformed the script.
    pub source_map: Option<FormulaSourceMap>,
}

// Cached for the lifetime of the process, keyed by raw script text, with no
// eviction. This assumes formula scripts come from a bounded set of app-defined
// templates (e.g. row_policy scripts configured per role), not from arbitrary
// unbounded user input -- feeding this cache directly from untrusted per-request
// text would grow it without limit. Add an eviction policy before doing that.
static SCRIPT_CACHE: LazyLock<RwLock<HashMap<String, Arc<ScriptDeps>>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Preprocess, compile, and analyze script, caching the result for future calls
/// with the same script text (see SCRIPT_CACHE for the caching tradeoff).
pub fn parse_and_cache(script: &str) -> Res<Arc<ScriptDeps>> {
    {
        let guard = SCRIPT_CACHE.read().map_err(|e| FormulaErr::Eval {
            inner: e.to_string(),
        })?;
        if let Some(cached) = guard.get(script) {
            return Ok(Arc::clone(cached));
        }
    }

    let (s, source_map) = preprocess_intl_template_with_map(script);

    let ast = FORMULA_ENGINE.compile(&*s).map_err(|e| {
        let hint = source_map
            .as_ref()
            .and_then(|sm| map_rhai_pos(sm, e.position()))
            .unwrap_or_default();
        FormulaErr::Compile {
            inner: format!("{e}{hint}"),
        }
    })?;

    let mut var_deps: HashSet<String> = HashSet::new();
    let mut local_vars: HashSet<String> = HashSet::new();
    let mut has_try_catch = false;

    ast.walk(&mut |nodes| {
        match nodes.last() {
            Some(ASTNode::Expr(Expr::Variable(data, _, _))) => {
                var_deps.insert(data.1.to_string());
            }
            Some(ASTNode::Stmt(Stmt::Var(data, _, _))) => {
                local_vars.insert(data.0.name.to_string());
            }
            Some(ASTNode::Stmt(Stmt::TryCatch(..))) => {
                has_try_catch = true;
            }
            _ => {}
        }
        true
    });

    // Rhai has no engine option to turn off try/catch parsing (unlike looping,
    // shadowing, anonymous functions, ...), so reject it here after the fact.
    // Formula scripts are meant to stay restricted expressions, not general
    // exception-handling control flow.
    if has_try_catch {
        return Err(FormulaErr::Compile {
            inner: "try/catch is not allowed in formula scripts".to_owned(),
        }
        .into());
    }

    let deps = Arc::new(ScriptDeps {
        ast: Arc::new(ast),
        var_deps: Arc::new(var_deps),
        local_vars: Arc::new(local_vars),
        source_map,
    });
    SCRIPT_CACHE
        .write()
        .map_err(|e| FormulaErr::Eval {
            inner: e.to_string(),
        })?
        .insert(script.to_owned(), Arc::clone(&deps));
    Ok(deps)
}

/// Translate a Rhai position in the preprocessed (generated) script back to the
/// original source position. Returns a hint like " (original 1:5)" or None.
pub fn map_rhai_pos(sm: &FormulaSourceMap, p: Position) -> Option<String> {
    let l = p.line()?;
    let c = p.position()?;
    if l <= 1 || c <= 1 {
        return None;
    }
    let l = (l - 1) as u32;
    let c = (c - 1) as u32;
    let t = sm.lookup_token(l, c)?;
    let l = t.get_src_line() + 1;
    let c = t.get_src_col() + 1;
    Some(format!(" (original {l}:{c})"))
}
