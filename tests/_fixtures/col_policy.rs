use grand_line::prelude::*;

/// Builds an allowed ColPolicyField with the given children.
pub const fn col_policy_field(children: ColPolicyFields) -> ColPolicyField {
    ColPolicyField {
        allow: true,
        children: Some(children),
    }
}
/// Builds an allowed ColPolicyField with no children, a leaf node in the allow-tree.
pub const fn col_policy_field_no_children() -> ColPolicyField {
    ColPolicyField {
        allow: true,
        children: None,
    }
}

/// Builds a single-entry ColPolicyFields map from key k to an allowed field with the given children.
pub fn col_policy_fields(k: String, children: ColPolicyFields) -> ColPolicyFields {
    hashmap! {
        k => col_policy_field(children),
    }
}
/// Builds a single-entry ColPolicyFields map from key k to an allowed field with no children.
pub fn col_policy_fields_no_children(k: String) -> ColPolicyFields {
    hashmap! {
        k => col_policy_field_no_children(),
    }
}

/// Builds a ColPolicyFields map with the "*" wildcard key, allowing all direct scalar children.
pub fn col_policy_fields_wildcard() -> ColPolicyFields {
    col_policy_fields_no_children("*".to_owned())
}
/// Builds a ColPolicyFields map with the "**" wildcard key, allowing the whole subtree at any depth.
pub fn col_policy_fields_wildcard_nested() -> ColPolicyFields {
    col_policy_fields_no_children("**".to_owned())
}

/// Combines the given input and output fields into a single ColPolicyOperation.
pub const fn col_policy_operation(inputs: ColPolicyField, output: ColPolicyField) -> ColPolicyOperation {
    ColPolicyOperation {
        inputs,
        output,
    }
}
/// Builds a single-entry ColPolicy map from key k to an operation built from inputs and output.
pub fn col_policy(k: String, inputs: ColPolicyField, output: ColPolicyField) -> ColPolicy {
    hashmap! {
        k => col_policy_operation(inputs, output),
    }
}

/// Builds a ColPolicy under the "*" operation key that allows every input and output field via a nested wildcard.
pub fn col_policy_wildcard() -> ColPolicy {
    let children = col_policy_fields_wildcard_nested();
    let field = col_policy_field(children);
    col_policy("*".to_owned(), field.clone(), field)
}

/// Builds a ColPolicy for key k that allows all input fields via a nested wildcard but restricts
/// the output to a single named child field child_k.
pub fn col_policy_with_children(k: &str, child_k: &str) -> ColPolicy {
    let inputs = col_policy_field(col_policy_fields_wildcard_nested());
    let output = col_policy_field(col_policy_fields_no_children(child_k.to_owned()));
    col_policy(k.to_owned(), inputs, output)
}
