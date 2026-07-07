use crate::prelude::*;

/// Entry point for the #[query] attribute macro, registers the annotated fn
/// as a root Query field with the inputs/output/body written as-is.
pub fn gen_query(attr: TokenStream, item: TokenStream) -> TokenStream {
    let a = parse_macro_input!(attr as AttrParse);
    let r = parse_macro_input!(item as ResolverTyItem);
    try_gen_query(a, r).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_query(a: AttrParse, r: ResolverTyItem) -> SynRes<TokenStream> {
    let a = a.into_inner::<ResolverTyAttr>("query")?;
    let (r, ty, name) = r.init("query", "", "")?;
    ResolverTy::g(ty, name, a, r)
}
