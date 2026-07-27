use crate::prelude::*;

/// Entry point for the #[update] attribute macro, builds an update mutation
/// resolver, defaulting inputs to id/data: <Model>Update and the output to the
/// model's Gql type, unless resolver_inputs/resolver_output opt out.
pub fn gen_update(attr: TokenStream, item: TokenStream) -> TokenStream {
    let a = parse_macro_input!(attr as AttrParse);
    let r = parse_macro_input!(item as ResolverTyItem);
    try_gen_update(a, r).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_update(attr: AttrParse, r: ResolverTyItem) -> SynRes<TokenStream> {
    let a = attr.into_inner::<CrudAttr>("update")?;
    let (mut r, ty, name) = r.init("mutation", "update", &a.model)?;
    a.validate(&r)?;

    if !a.resolver_inputs {
        let data = name.to_string().to_pascal_case().ts2_or_err()?;
        r.inputs = quote! {
            id: String,
            data: #data,
        };
    }

    if !a.resolver_output {
        let model = a.model.ts2_or_err()?;
        let output = ty_gql(&model)?;
        r.output = quote!(#output);

        let body = r.body;
        let am = ty_am(&model)?;

        let into = if a.ra.has_auth() {
            quote!(into_am(ctx).await?)
        } else {
            quote!(into_am_without_ctx())
        };

        let filter = ty_filter(&model)?;
        let (authz_row, authz_row_def) = gen_authz_row_def(&filter, a.ra.authz_row);
        let authz_err = gen_authz_err(a.ra.authz_row);

        r.body = quote! {
            #authz_row_def
            #model::gql_mutation_check_id(
                ctx,
                tx,
                &id,
                #authz_row.clone(),
                #authz_err,
            )
            .await?;
            let am: AmWrapper<AmUpdate, #model, #am> = {
                #body
            };
            #model::gql_update(
                ctx,
                tx,
                &id,
                am.#into,
                #authz_row,
                #authz_err,
            )
            .await?
        }
    }

    ResolverTy::g(ty, name, a.ra, r)
}
