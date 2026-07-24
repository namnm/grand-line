use grand_line::prelude::*;

/// A named permission bundle: which columns and rows an assigned actor may access.
#[model]
pub struct Role {
    pub name: String,
    /// Groups multiple roles into a realm, e.g. "org" or "system".
    pub realm: String,
    /// Map to ColPolicy, checked once at the operation root.
    pub col_policy: JsonValue,
    /// Map to RowPolicy, checked lazily per relation field.
    pub row_policy: JsonValue,
    /// None for realm-wide roles not tied to a single org (e.g. "system").
    pub org_id: Option<String>,
}
