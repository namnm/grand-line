use grand_line::prelude::*;

/// Builds a single-entry RowPolicy map from field path k to the given script.
pub fn row_policy(k: String, script: String) -> RowPolicy {
    hashmap! {
        k => script,
    }
}
