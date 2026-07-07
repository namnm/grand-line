/// Runtime limits applied to each Rhai evaluation engine.
///
/// All fields default to the values used historically. Override individual
/// limits by constructing `FormulaOptions { max_operations: 500, ..Default::default() }`.
#[derive(Clone)]
pub struct FormulaOptions {
    pub max_array_size: usize,
    pub max_call_levels: usize,
    /// Depth limit for the outer expression tree.
    pub max_expr_depth: usize,
    /// Depth limit for function bodies (inner expression tree).
    pub max_fn_expr_depth: usize,
    /// Max number of user-defined script functions (0 = none allowed).
    pub max_functions: usize,
    pub max_map_size: usize,
    pub max_operations: u64,
    pub max_string_size: usize,
    pub max_strings_interned: usize,
    pub max_variables: usize,
}

impl Default for FormulaOptions {
    fn default() -> Self {
        Self {
            max_array_size: 50,
            max_call_levels: 8,
            max_expr_depth: 10,
            max_fn_expr_depth: 2,
            max_functions: 0,
            max_map_size: 64,
            max_operations: 1_000,
            max_string_size: 1024,
            max_strings_interned: 32,
            max_variables: 16,
        }
    }
}
