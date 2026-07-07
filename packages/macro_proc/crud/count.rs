use crate::prelude::*;

/// Entry point for the #[count] attribute macro, builds a count query resolver
/// returning u64, defaulting inputs to filter (plus include_deleted when
/// enabled), unless resolver_inputs/resolver_output opt out.
pub fn gen_count(attr: TokenStream, item: TokenStream) -> TokenStream {
    let a = parse_macro_input!(attr as AttrParse);
    let r = parse_macro_input!(item as ResolverTyItem);
    try_gen_count(a, r).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_count(attr: AttrParse, r: ResolverTyItem) -> SynRes<TokenStream> {
    let a = attr.into_inner::<CrudAttr>("count")?;
    let (mut r, ty, name) = r.init("query", "count", &a.model)?;
    a.validate(&r)?;

    let filter = ty_filter(&a.model)?;

    if !a.resolver_inputs {
        r.inputs = quote! {
            filter: Option<#filter>,
        };
        r.inputs = push_include_deleted(r.inputs, a.ra.include_deleted);
    }

    if !a.resolver_output {
        r.output = quote!(u64);

        let body = ensure_default_tail(r.body)?;
        let model = a.model.ts2_or_err()?;

        let extra = unique_ident();
        let authz_row = gen_authz_row(&filter, a.ra.authz_row);
        let include_deleted = get_include_deleted(!a.resolver_inputs && a.ra.include_deleted);

        r.body = quote! {
            let #extra: Count = #body;
            let #extra = #extra.add_option(#authz_row);
            #model::gql_count(
                ctx,
                tx,
                filter,
                #include_deleted,
                #extra,
            )
            .await?
        };
    }

    ResolverTy::g(ty, name, a.ra, r)
}
