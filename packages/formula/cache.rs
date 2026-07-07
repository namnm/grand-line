use crate::engine::FORMULA_ENGINE;
use crate::err::FormulaErr;
use crate::preprocess::preprocess_intl_template_with_map;
use _core::prelude::*;
use rhai::{AST, ASTNode, Expr, Position, Stmt};
use sourcemap::SourceMap;
use std::sync::RwLock;

pub struct ScriptDeps {
    pub ast: Arc<AST>,
    /// Variables referenced in the script (`Expr::Variable` nodes).
    pub var_deps: Arc<HashSet<String>>,
    /// Variables declared with `let` in the script (`Stmt::Var` nodes).
    /// Excluded from validation -- locally defined, not external scope requirements.
    pub local_vars: Arc<HashSet<String>>,
    /// Source map from preprocessed script positions back to original positions.
    /// Present only when `preprocess_intl_template_with_map` transformed the script.
    pub source_map: Option<SourceMap>,
}

static SCRIPT_CACHE: LazyLock<RwLock<HashMap<String, Arc<ScriptDeps>>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn parse_and_cache(script: &str) -> Res<Arc<ScriptDeps>> {
    {
        let guard = SCRIPT_CACHE.read().map_err(|e| FormulaErr::Eval(e.to_string()))?;
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
        FormulaErr::Compile(format!("{e}{hint}"))
    })?;

    let mut var_deps: HashSet<String> = HashSet::new();
    let mut local_vars: HashSet<String> = HashSet::new();

    ast.walk(&mut |nodes| {
        match nodes.last() {
            Some(ASTNode::Expr(Expr::Variable(data, _, _))) => {
                var_deps.insert(data.1.to_string());
            }
            Some(ASTNode::Stmt(Stmt::Var(data, _, _))) => {
                local_vars.insert(data.0.name.to_string());
            }
            _ => {}
        }
        true
    });

    let deps = Arc::new(ScriptDeps {
        ast: Arc::new(ast),
        var_deps: Arc::new(var_deps),
        local_vars: Arc::new(local_vars),
        source_map,
    });
    SCRIPT_CACHE
        .write()
        .map_err(|e| FormulaErr::Eval(e.to_string()))?
        .insert(script.to_owned(), Arc::clone(&deps));
    Ok(deps)
}

/// Translate a Rhai position in the preprocessed (generated) script back to the
/// original source position. Returns a hint like " (original 1:5)" or None.
pub fn map_rhai_pos(sm: &SourceMap, p: Position) -> Option<String> {
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
