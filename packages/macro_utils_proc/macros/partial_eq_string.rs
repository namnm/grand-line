use crate::prelude::*;

/// Implementation of the PartialEqString derive macro, see macro_utils_proc::partial_eq_string.
pub fn gen_partial_eq_string(input: TokenStream) -> TokenStream {
    let ty = parse_macro_input!(input as DeriveInput).ident;
    quote! {
        impl PartialEq<&str> for #ty {
            fn eq(&self, other: &&str) -> bool {
                self.as_ref() == *other
            }
        }
        impl PartialEq<String> for #ty {
            fn eq(&self, other: &String) -> bool {
                self.as_ref() == other.as_str()
            }
        }
        impl PartialEq<#ty> for &str {
            fn eq(&self, other: &#ty) -> bool {
                *self == other.as_ref()
            }
        }
        impl PartialEq<#ty> for String {
            fn eq(&self, other: &#ty) -> bool {
                self.as_str() == other.as_ref()
            }
        }
    }
    .into()
}
