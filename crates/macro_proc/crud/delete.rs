use crate::prelude::*;

/// Entry point for the #[delete] attribute macro, builds a delete mutation
/// resolver, defaulting inputs to id (plus permanent when permanent isenabled)
/// and the output to the model's Gql type, unless resolver_inputs/resolver_output opt out.
pub fn gen_delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    let a = parse_macro_input!(attr as AttrParse);
    let r = parse_macro_input!(item as ResolverTyItem);
    try_gen_delete(a, r).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_delete(attr: AttrParse, r: ResolverTyItem) -> SynRes<TokenStream> {
    let a = attr.into_inner::<CrudAttr>("delete")?;
    let (mut r, ty, name) = r.init("mutation", "delete", &a.model)?;
    a.validate(&r)?;

    if !a.resolver_inputs {
        r.inputs = quote! {
            id: String,
        };
        if a.permanent {
            let inputs = r.inputs;
            r.inputs = quote! {
                #inputs
                permanent: Option<bool>,
            }
        }
    }

    if !a.resolver_output {
        let model = a.model.ts2_or_err()?;
        let output = ty_gql(&model)?;
        r.output = quote!(#output);

        let body = r.body;
        let permanent = if !a.resolver_inputs && a.permanent {
            quote!(permanent)
        } else {
            quote!(None)
        };

        let filter = ty_filter(&model)?;
        let (authz_row, authz_row_def) = gen_authz_row_def(&filter, a.ra.authz_row);
        let authz_err = gen_authz_err(a.ra.authz_row);

        let by_id = gen_auth_by_id();

        let g = unique_ident();
        let subscription_queue = gen_subscription_queue(&model, &quote!(Delete), &quote!(&id), a.ra.publish);

        r.body = quote! {
            #authz_row_def
            #model::gql_mutation_check_id(
                ctx,
                db,
                &id,
                #authz_row.clone(),
                #authz_err,
            )
            .await?;

            #body
            let #g = #model::gql_delete(
                ctx,
                db,
                &id,
                #permanent,
                #authz_row,
                #authz_err,
                #by_id,
            )
            .await?;
            #subscription_queue
            #g
        };
    }

    ResolverTy::g(ty, name, a.ra, r)
}
