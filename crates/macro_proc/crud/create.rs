use crate::prelude::*;

/// Entry point for the #[create] attribute macro, builds a create mutation
/// resolver, defaulting inputs to data: <Model>Create and the output to the
/// model's Gql type unless resolver_inputs/resolver_output opt out.
pub fn gen_create(attr: TokenStream, item: TokenStream) -> TokenStream {
    let a = parse_macro_input!(attr as AttrParse);
    let r = parse_macro_input!(item as ResolverTyItem);
    try_gen_create(a, r).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_create(attr: AttrParse, r: ResolverTyItem) -> SynRes<TokenStream> {
    let a = attr.into_inner::<CrudAttr>("create")?;
    let (mut r, ty, name) = r.init("mutation", "create", &a.model)?;
    a.validate(&r)?;

    if !a.resolver_inputs {
        let data = name.to_string().to_pascal_case().ts2_or_err()?;
        r.inputs = quote!(data: #data);
    }

    if !a.resolver_output {
        let model = a.model.ts2_or_err()?;
        let output = ty_gql(&model)?;
        r.output = quote!(#output);

        let body = r.body;
        let am = ty_am(&model)?;

        let exec = if a.ra.has_auth() {
            quote!(exec(ctx))
        } else {
            quote!(exec_without_ctx(tx))
        };

        r.body = quote! {
            let am: AmWrapper<AmCreate, #model, #am> = {
                #body
            };
            am.#exec.await?.into_gql(ctx).await?
        }
    }

    ResolverTy::g(ty, name, a.ra, r)
}
