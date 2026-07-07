use super::prelude::*;

/// One gql-requested field resolved to its sql column or expression, produced by gql_look_ahead.
pub struct LookaheadX<E>
where
    E: EntityX,
{
    /// Rust snake_case field name, as used in gql_select, gql_cols and gql_exprs.
    pub c: &'static str,
    /// Sql column for c, set when the field maps directly to a table column.
    pub col: Option<E::C>,
    /// Sql expression for c, set when the field maps to a computed expression instead of a plain column.
    pub expr: Option<SimpleExpr>,
}
