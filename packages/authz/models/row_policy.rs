use crate::prelude::*;

/// RowPolicy is a flat map from field path to a script string.
/// Evaluated lazily inside resolvers that call authz_row(), not at the operation root.
pub type RowPolicy = HashMap<String, String>;
