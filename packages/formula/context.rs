use crate::prelude::*;

pub struct FormulaCtx {
    /// Dependency graph of async resolvers: nodes are resolved in topological
    /// order before Rhai eval. Defaults to a graph containing only `now`.
    pub row_graph: FormulaDepGraph,
    /// Per-eval function registration for row formulas. Called once per eval to
    /// register functions on a fresh Rhai engine. Use `register_db_fns` to add
    /// `db_find_one` / `db_find_many`. Use `None` when no functions are needed.
    pub row_register_fns: Option<FormulaEngineFns>,
    /// Runtime limits for row formula evaluation engines.
    pub row_formula_opts: FormulaOptions,
    /// Default locale tag used when evaluating row formulas.
    pub row_locale: &'static str,
}
impl Default for FormulaCtx {
    fn default() -> Self {
        Self {
            row_graph: Default::default(),
            row_register_fns: Default::default(),
            row_formula_opts: Default::default(),
            row_locale: "en",
        }
    }
}
