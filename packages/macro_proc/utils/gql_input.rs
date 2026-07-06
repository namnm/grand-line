use crate::prelude::*;

pub fn gen_gql_input(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = Into::<Ts2>::into(attr);
    let item = Into::<Ts2>::into(item);

    quote! {
        #[serde_with::skip_serializing_none]
        #[derive(
            Debug,
            Clone,
            Default,
            Serialize,
            Deserialize,
            InputObject,
        )]
        #attr
        #item
    }
    .into()
}
