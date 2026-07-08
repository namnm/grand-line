use crate::prelude::*;

struct Item {
    model: Ident,
    fields: Punctuated<Ident, Token![,]>,
}

impl Parse for Item {
    fn parse(input: ParseStream) -> SynRes<Self> {
        let model = input.parse::<Ident>()?;
        let c;
        bracketed!(c in input);
        let fields = c.parse_terminated(Ident::parse, Token![,])?;
        Ok(Self {
            model,
            fields,
        })
    }
}

/// Need to use proc macro to concat model name with `OrderBy`.
/// Can be simpler general macro with paste, but we dont want to export paste.
pub fn gen_order_by(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as Item);
    try_gen_order_by(item).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_order_by(item: Item) -> SynRes<TokenStream> {
    let order_by = ty_order_by(item.model)?;
    let paths = item.fields.iter().map(|f| quote!(#order_by::#f,));
    Ok(quote!(vec![#(#paths)*]).into())
}
