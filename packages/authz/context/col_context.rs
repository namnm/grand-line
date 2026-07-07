use crate::prelude::*;

/// Checks a GraphQL field's arguments and selected output against a role's
/// ColPolicyField allow-tree.
pub trait AuthzColContext<'a>
where
    Self: ImplContext<'a>,
{
    /// Entry point: verify every selected field in the GraphQL response against output.
    fn authz_col_check_output(&self, output: &ColPolicyField) -> bool {
        check_output(self.field_impl(), output)
    }

    /// Entry point: verify every argument in the current GraphQL field against inputs.
    fn authz_col_check_inputs(&self, inputs: &ColPolicyField) -> bool {
        let Ok(pairs) = self.field_impl().arguments() else {
            return false;
        };
        for (k, v) in pairs {
            if !check_inputs_tree(inputs, k.as_str(), &v) {
                return false;
            }
        }
        true
    }
}

impl<'a> AuthzColContext<'a> for Context<'a> {
}

// ---------------------------------------------------------------------------
// Allow-tree matching helpers
// ---------------------------------------------------------------------------

/// Check a single key-value argument pair against parent.
/// Dispatches on value shape:
///   List   -- each element is checked against the child policy (check_inputs_value).
///   Object -- each field of the object recurses back into check_inputs_tree.
///   scalar -- the child key must exist and be allowed, or parent wildcard(*) applies.
fn check_inputs_tree(parent: &ColPolicyField, child_k: &str, child_v: &GraphQLValue) -> bool {
    if parent.wildcard_nested() {
        return true;
    }
    let child = parent.children.as_ref().and_then(|m| m.get(child_k));

    match child_v {
        GraphQLValue::List(list) => {
            let Some(child) = child else {
                return false;
            };
            list.iter().all(|v| check_inputs_value(child, v))
        }
        GraphQLValue::Object(object) => {
            let Some(child) = child else {
                return false;
            };
            object.iter().all(|(k, v)| check_inputs_tree(child, k.as_str(), v))
        }
        _ => parent.wildcard() || child.is_some_and(|p| p.allow),
    }
}

/// Check a value that is already known to belong to an allowed child.
/// Handles the case where a list element is itself an object or another list.
fn check_inputs_value(child: &ColPolicyField, child_v: &GraphQLValue) -> bool {
    if child.wildcard_nested() {
        return true;
    }
    match child_v {
        GraphQLValue::List(list) => list.iter().all(|v| check_inputs_value(child, v)),
        GraphQLValue::Object(object) => object.iter().all(|(k, v)| {
            let grand = child.children.as_ref().and_then(|m| m.get(k.as_str()));
            let Some(grand) = grand else {
                return false;
            };
            check_inputs_value(grand, v)
        }),
        _ => child.allow,
    }
}

/// Recursively walk the selection set.
/// Nested selections (objects/relations) must have an explicit child entry.
/// Leaf fields (scalars) may be covered by the parent wildcard(*).
fn check_output(field: SelectionField<'_>, parent: &ColPolicyField) -> bool {
    if parent.wildcard_nested() {
        return true;
    }
    for sub in field.selection_set() {
        let child_k = sub.name();
        let child = parent.children.as_ref().and_then(|m| m.get(child_k));
        if sub.selection_set().next().is_some() {
            // Nested object/relation: must have an explicit entry and pass recursively.
            let Some(child) = child else {
                return false;
            };
            if !check_output(sub, child) {
                return false;
            }
        } else {
            // Leaf scalar: allowed if explicitly listed, covered by wildcard, or a built-in.
            let allow = parent.wildcard() || ALLOW_BUILT_IN.contains(child_k) || child.is_some_and(|p| p.allow);
            if !allow {
                return false;
            }
        }
    }
    // Empty selection set means a scalar resolved by a parent recursive call -- allow.
    true
}

static ALLOW_BUILT_IN: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut m = HashSet::new();
    m.insert("__typename");
    m
});
