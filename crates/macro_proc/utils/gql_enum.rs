use crate::prelude::*;

/// Entry point for the #[gql_enum] attribute macro, attaches the derives
/// needed for a GraphQL-facing Copy enum (Debug, Clone, Eq, PartialEq, Copy,
/// Serialize, Deserialize, async-graphql Enum).
pub fn gen_gql_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = Into::<Ts2>::into(attr);
    let item = Into::<Ts2>::into(item);

    quote! {
        #[derive(
            Debug,
            Clone,
            Eq,
            PartialEq,
            Copy,
            Serialize,
            Deserialize,
            Enum,
        )]
        #attr
        #item
    }
    .into()
}
