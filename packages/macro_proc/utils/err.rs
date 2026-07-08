use crate::prelude::*;

pub fn gen_grand_line_err(_: TokenStream, item: TokenStream) -> TokenStream {
    let item = Into::<Ts2>::into(item);

    quote! {
        #[derive(
            Debug,
            ThisErr,
            GrandLineErrDerive,
        )]
        #item
    }
    .into()
}

pub fn gen_grand_line_err_derive(item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    let name = &item.ident;

    let Data::Enum(d) = &item.data else {
        let msg = "GrandLineErrDerive only support enum with ThisErr";
        return SynErr::new_spanned(&item.ident, msg).to_compile_error().into();
    };

    let mut codes = Vec::new();
    let mut clients = Vec::new();

    for v in &d.variants {
        let v_ident = &v.ident;

        let field = match &v.fields {
            Fields::Unit => quote! { Self::#v_ident },
            Fields::Unnamed(_) => quote! { Self::#v_ident ( .. ) },
            Fields::Named(_) => quote! { Self::#v_ident { .. } },
        };

        let mut code = None;
        for attr in &v.attrs {
            if attr.path().is_ident("code")
                && let Meta::NameValue(MetaNameValue {
                    value,
                    ..
                }) = &attr.meta
                && let Expr::Lit(ExprLit {
                    lit: Lit::Str(s),
                    ..
                }) = value
            {
                code = Some(s.value());
            }
        }
        let code = code.unwrap_or_else(|| v_ident.to_string());
        codes.push(quote! {
            #field => #code,
        });

        let client = v.attrs.iter().any(|a| a.path().is_ident("client"));
        clients.push(quote! {
            #field => #client,
        });
    }

    quote! {
        impl GrandLineErrImpl for #name {
            fn code(&self) -> &'static str {
                match self {
                    #(#codes)*
                }
            }
            fn client(&self) -> bool {
                match self {
                    #(#clients)*
                }
            }
        }
        impl From<#name> for GrandLineErr {
            fn from(v: #name) -> Self {
                GrandLineErr(Arc::new(v))
            }
        }
    }
    .into()
}
