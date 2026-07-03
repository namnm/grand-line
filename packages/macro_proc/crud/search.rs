use crate::prelude::*;

pub fn gen_search(attr: TokenStream, item: TokenStream) -> TokenStream {
    let a = parse_macro_input!(attr as AttrParse);
    let r = parse_macro_input!(item as ResolverTyItem);
    try_gen_search(a, r).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_search(attr: AttrParse, r: ResolverTyItem) -> SynRes<TokenStream> {
    let a = attr.into_inner::<CrudAttr>("search")?;
    let (mut r, ty, name) = r.init("query", "search", &a.model)?;
    a.validate(&r)?;

    let filter = ty_filter(&a.model)?;
    let order_by = ty_order_by(&a.model)?;

    if !a.resolver_inputs {
        r.inputs = quote! {
            filter: Option<#filter>,
            order_by: Option<Vec<#order_by>>,
            page: Option<Pagination>,
        };
        r.inputs = push_include_deleted(r.inputs, a.ra.include_deleted);
    }

    if !a.resolver_output {
        let output = ty_gql(&a.model)?;
        r.output = quote!(Vec<#output>);

        let body = ensure_default_tail(r.body)?;
        let model = a.model.ts2_or_err()?;

        let extra = unique_ident();
        let authz_row = gen_authz_row(&filter, a.ra.authz_row);
        let include_deleted = get_include_deleted(!a.resolver_inputs && a.ra.include_deleted);

        r.body = quote! {
            let #extra: Search<#order_by> = #body;
            let #extra = #extra.add_option(#authz_row);
            #model::gql_search(
                ctx,
                tx,
                filter,
                order_by,
                page,
                #include_deleted,
                #extra,
            )
            .await?
        };
    }

    ResolverTy::g(ty, name, a.ra, r)
}
