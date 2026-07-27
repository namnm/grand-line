mod macros;

#[allow(ambiguous_glob_reexports, dead_code, unused_imports)]
mod prelude {
    pub use crate::macros::*;
    pub use _utils::*;
    pub use proc_macro::TokenStream;
    use_common_macro_utils!();
    use_common_std!();
}
use crate::prelude::*;

/// Derives PartialEq<&str> and PartialEq<String> (and the reverse impls),
/// comparing through the type's AsRef<str> implementation.
#[proc_macro_derive(PartialEqString)]
pub fn partial_eq_string(input: TokenStream) -> TokenStream {
    gen_partial_eq_string(input)
}

/// Attribute macro for a named-field struct that generates a FIELDS constant
/// and a FIELD_name constant per field, use #[field_names(skip, key_only)]
/// on a field to exclude it or mark it as a virtual, non-stored key.
#[proc_macro_attribute]
pub fn field_names(attr: TokenStream, input: TokenStream) -> TokenStream {
    gen_field_names(attr, input)
}

/// Function-like macro that expands to a public default_NAME function
/// returning whether the NAME cargo feature is enabled.
#[proc_macro]
pub fn attr_default_flag(input: TokenStream) -> TokenStream {
    gen_attr_default_flag(input)
}
