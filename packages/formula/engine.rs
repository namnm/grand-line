use crate::opts::FormulaOptions;
use rhai::{Engine, ImmutableString, Map as RhaiMap, OptimizationLevel};
use std::sync::LazyLock;

pub static BUILTIN_SCOPE_VARS: &[&str] = &["current_user", "current_org"];

fn intl_substitute(template: &str, vars: &RhaiMap) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        rest = &rest[open + 1..];
        if let Some(close) = rest.find('}') {
            let key = &rest[..close];
            rest = &rest[close + 1..];
            if let Some(val) = vars.get(key) {
                out.push_str(&val.to_string());
            } else {
                out.push('{');
                out.push_str(key);
                out.push('}');
            }
        } else {
            out.push('{');
        }
    }
    out.push_str(rest);
    out
}

pub fn make_base_engine(opts: &FormulaOptions) -> Engine {
    let mut engine = Engine::new_raw();
    // 1-arg: template with no placeholders -- return as-is.
    engine.register_fn("intl", |s: ImmutableString| -> String { s.to_string() });
    // 2-arg: substitute {varName} placeholders with values from the map.
    engine.register_fn("intl", |s: ImmutableString, vars: RhaiMap| -> String {
        intl_substitute(&s, &vars)
    });
    engine
        .set_allow_anonymous_fn(false)
        .set_allow_if_expression(true)
        .set_allow_loop_expressions(false)
        .set_allow_looping(false)
        .set_allow_shadowing(false)
        .set_allow_statement_expression(false)
        .set_allow_switch_expression(false)
        .set_fail_on_invalid_map_property(true)
        .set_fast_operators(true)
        .set_max_array_size(opts.max_array_size)
        .set_max_call_levels(opts.max_call_levels)
        .set_max_expr_depths(opts.max_expr_depth, opts.max_fn_expr_depth)
        .set_max_functions(opts.max_functions)
        .set_max_map_size(opts.max_map_size)
        .set_max_operations(opts.max_operations)
        .set_max_string_size(opts.max_string_size)
        .set_max_strings_interned(opts.max_strings_interned)
        .set_max_variables(opts.max_variables)
        // Simple: folds constant arithmetic but never calls registered functions
        // during compilation, so per-eval registered fns are not pre-evaluated.
        .set_optimization_level(OptimizationLevel::Simple);
    engine
}

/// Compile-only engine: limits are irrelevant during parsing; defaults are fine.
pub static FORMULA_ENGINE: LazyLock<Engine> = LazyLock::new(|| make_base_engine(&FormulaOptions::default()));
