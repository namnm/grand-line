use crate::prelude::*;

/// A named permission bundle: which columns and rows an assigned actor may access.
#[model]
pub struct Role {
    pub name: String,
    /// To group multiple roles into a realm. For example: system, org, public...
    pub realm: String,
    /// Map to ColPolicy hash map, per operation, check once at operation root
    /// recursively all columns in request.
    /// The map is nested, but non-recursive to be json safe.
    pub col_policy: JsonValue,
    /// Map to RowPolicy hash map, per field, to dynamically run dsl script and return json.
    /// The json will be parsed as Filter and become sea orm where condition.
    pub row_policy: JsonValue,
    /// Can be none if this role is not related to an org. For example system or built in roles.
    pub org_id: Option<String>,
}
