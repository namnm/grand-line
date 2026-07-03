use crate::prelude::*;

/// ColPolicy is a map from operation name (or "*" wildcard) to its per-operation policy.
/// Checked once at the operation root resolver before any DB work.
pub type ColPolicy = HashMap<String, ColPolicyOperation>;

#[gql_input]
pub struct ColPolicyOperation {
    /// Controls which GraphQL arguments callers may pass.
    pub inputs: ColPolicyField,
    /// Controls which GraphQL fields callers may select in the response.
    pub output: ColPolicyField,
}

/// A node in the allow-tree.  Both inputs and output share this shape.
///
/// Wildcard semantics (stored as special keys in children):
///   "*"  -- allow all direct scalar children without explicit entries.
///   "**" -- allow all descendants at any depth (short-circuits the whole subtree).
#[gql_input]
pub struct ColPolicyField {
    pub allow: bool,
    pub children: Option<ColPolicyFields>,
}
pub type ColPolicyFields = HashMap<String, ColPolicyField>;

impl ColPolicyField {
    /// True when children["*"].allow -- direct scalars are allowed without explicit entries.
    pub fn wildcard(&self) -> bool {
        self.children
            .as_ref()
            .map(|m| m.get("*"))
            .unwrap_or(None)
            .is_some_and(|p| p.allow)
    }

    /// True when children["**"].allow -- entire subtree is allowed, skip further checks.
    pub fn wildcard_nested(&self) -> bool {
        self.children
            .as_ref()
            .map(|m| m.get("**"))
            .unwrap_or(None)
            .is_some_and(|p| p.allow)
    }
}
