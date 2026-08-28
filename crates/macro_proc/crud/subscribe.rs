use crate::prelude::*;

/// Entry point for the #[subscribe] attribute macro, builds a root subscription
/// field streaming every change to the model, defaulting the input to filter and
/// the output to a generated <Model>Event, unless resolver_inputs/resolver_output
/// opt out. The body contributes a Detail, same as #[detail], applied to every
/// event before it reaches the client.
pub fn gen_subscribe(attr: TokenStream, item: TokenStream) -> TokenStream {
    let a = parse_macro_input!(attr as AttrParse);
    let r = parse_macro_input!(item as ResolverTyItem);
    try_gen_subscribe(a, r).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_subscribe(attr: AttrParse, r: ResolverTyItem) -> SynRes<TokenStream> {
    let a = attr.into_inner::<CrudAttr>("subscribe")?;
    let (r, ty, name) = r.init("subscription", "changed", &a.model)?;
    a.validate(&r)?;

    let model = a.model.ts2_or_err()?;
    let gql = ty_gql(&model)?;
    let filter = ty_filter(&model)?;
    let gql_name = &r.gql_name;
    // Named after the field, not the model, so two subscriptions on one model
    // do not collide on a single event type.
    let event = format!("{}Event", gql_name.to_pascal_case()).ts2_or_err()?;
    let gql_event = event.to_string();

    let allow_permanent_delete = a.allow_permanent_delete;
    let body = ensure_default_tail(r.body)?;
    let checks = a.ra.check.iter().map(CheckAttr::call);
    let extra = unique_ident();
    let item = unique_ident();

    let m = ty.to_string().to_snake_case().ts2_or_err()?;
    let g = quote! {
        mod #m {
            use super::*;

            #[derive(SimpleObject)]
            #[graphql(name = #gql_event)]
            pub struct #event {
                /// Set when a row was created, null on every other change.
                pub created: Option<#gql>,
                /// Set when a row was updated, null on every other change.
                pub updated: Option<#gql>,
                /// Set when a row was deleted, null on every other change. Only the
                /// id resolves once the row is gone for good.
                pub deleted: Option<#gql>,
            }

            #[derive(Default)]
            pub struct #ty;
            #[Subscription]
            impl #ty {
                #[graphql(name = #gql_name)]
                async fn #name(
                    &self,
                    ctx: &Context<'_>,
                    filter: Option<#filter>,
                ) -> Result<impl Stream<Item = Res<#event>>, GrandLineErr> {
                    #(#checks)*
                    let #extra: Detail = {
                        #body
                    };
                    // The guards and the body are the only things here that touch
                    // the request transaction, and the stream outlives the request,
                    // so the connection goes back to the pool before it starts.
                    ctx.tx_finish().await?;
                    let r = subscription_stream::<#model>(ctx, filter, #extra, #allow_permanent_delete)?.map(|r| {
                        r.map(|#item| {
                            let (created, updated, deleted) = #item.split();
                            #event {
                                created,
                                updated,
                                deleted,
                            }
                        })
                    });
                    Ok(r)
                }
            }
        }
        pub use #m::{#event, #ty};
    };

    #[cfg(feature = "debug_macro")]
    debug_macro(gql_name, &g);

    Ok(g.into())
}
