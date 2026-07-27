use crate::prelude::*;

/// Crud macro kinds that support an include_deleted input (search, count, detail).
pub static TY_INCLUDE_DELETED: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    set.insert(MacroTy::Search.to_string());
    set.insert(MacroTy::Count.to_string());
    set.insert(MacroTy::Detail.to_string());
    set
});

/// Appends an include_deleted: Option<bool> field to inputs when enabled,
/// otherwise returns inputs unchanged.
pub fn push_include_deleted(inputs: Ts2, enable: bool) -> Ts2 {
    if enable {
        quote! {
            #inputs
            include_deleted: Option<bool>,
        }
    } else {
        inputs
    }
}
/// Token to pass as the include_deleted argument to the gql_* call: the
/// include_deleted input variable when enabled, or None when disabled.
pub fn get_include_deleted(enable: bool) -> Ts2 {
    if enable {
        quote!(include_deleted)
    } else {
        quote!(None)
    }
}
