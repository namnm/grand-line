use crate::prelude::*;

#[gql_input]
pub struct PolicyOperation {
    pub inputs: PolicyField,
    pub output: PolicyField,
    // rhai expression evaluated to a filter map, e.g. #{ assigned_to_id_eq: current_user }
    pub row: Option<String>,
}
pub type PolicyOperations = HashMap<String, PolicyOperation>;

#[gql_input]
pub struct PolicyField {
    pub allow: bool,
    pub children: Option<PolicyFields>,
}
pub type PolicyFields = HashMap<String, PolicyField>;

impl PolicyField {
    pub(crate) fn wildcard(&self) -> bool {
        self.children
            .as_ref()
            .map(|m| m.get("*"))
            .unwrap_or(None)
            .is_some_and(|p| p.allow)
    }
    pub(crate) fn wildcard_nested(&self) -> bool {
        self.children
            .as_ref()
            .map(|m| m.get("**"))
            .unwrap_or(None)
            .is_some_and(|p| p.allow)
    }
}

pub fn policy_check_inputs(ctx: &Context<'_>, inputs: &PolicyField) -> bool {
    let Ok(pairs) = ctx.field().arguments() else {
        return false;
    };

    for (k, v) in pairs {
        if !check_inputs_tree(inputs, k.as_str(), &v) {
            return false;
        }
    }

    true
}

fn check_inputs_tree(parent: &PolicyField, child_k: &str, child_v: &GraphQLValue) -> bool {
    if parent.wildcard_nested() {
        return true;
    }
    let child = parent.children.as_ref().map(|m| m.get(child_k)).unwrap_or(None);

    match child_v {
        GraphQLValue::List(list) => {
            let Some(child) = child else {
                return false;
            };
            for child_v in list {
                if !check_inputs_value(child, child_v) {
                    return false;
                }
            }
        }
        GraphQLValue::Object(object) => {
            let Some(child) = child else {
                return false;
            };
            for (grand_k, grand_v) in object {
                if !check_inputs_tree(child, grand_k.as_str(), grand_v) {
                    return false;
                }
            }
        }
        _ => {
            let allow = parent.wildcard() || child.is_some_and(|p| p.allow);
            if !allow {
                return false;
            }
        }
    }

    true
}

fn check_inputs_value(child: &PolicyField, child_v: &GraphQLValue) -> bool {
    if child.wildcard_nested() {
        return true;
    }

    match child_v {
        GraphQLValue::List(list) => {
            for child_v in list {
                if !check_inputs_value(child, child_v) {
                    return false;
                }
            }
        }
        GraphQLValue::Object(object) => {
            for (grand_k, grand_v) in object {
                let grand = child.children.as_ref().map(|m| m.get(grand_k.as_str())).unwrap_or(None);
                let Some(grand) = grand else {
                    return false;
                };
                if !check_inputs_value(grand, grand_v) {
                    return false;
                }
            }
        }
        _ => {
            if !child.allow {
                return false;
            }
        }
    }

    true
}

pub fn policy_check_output(ctx: &Context<'_>, output: &PolicyField) -> bool {
    let field = ctx.field();
    check_output(field, output)
}

fn check_output(field: SelectionField<'_>, parent: &PolicyField) -> bool {
    if parent.wildcard_nested() {
        return true;
    }

    for sub in field.selection_set() {
        let child_k = sub.name();
        let child = parent.children.as_ref().map(|m| m.get(child_k)).unwrap_or(None);

        let has_children = sub.selection_set().next().is_some();
        if has_children {
            let Some(child) = child else {
                return false;
            };
            let allow = check_output(sub, child);
            if !allow {
                return false;
            }
        } else {
            let allow = parent.wildcard() || ALLOW_BUILT_IN.contains(child_k) || child.is_some_and(|p| p.allow);
            if !allow {
                return false;
            }
        }
    }

    // if the selection set is empty which mean primitive type such as String Int
    // it should be allowed here since we already checked in the previous rescursive
    true
}

static ALLOW_BUILT_IN: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let mut m = HashSet::new();
    m.insert("__typename");
    m
});
