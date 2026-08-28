use crate::prelude::*;

/// Assembles a resolver method into its own module with a dedicated zero-sized
/// struct implementing #[Object], this is the shared codegen backend for
/// #[query], #[mutation], and all the crud macros.
pub struct ResolverTy {
    ty: Ts2,
    name: Ts2,
    ra: ResolverTyAttr,
    item: ResolverTyItem,
}

impl ResolverTy {
    /// Builds the final module + struct + #[Object] impl token stream for one
    /// resolver, this is the common tail called by every resolver-generating
    /// attribute macro after it has filled in ty/name/ra/item.
    pub fn g(ty: Ts2, name: Ts2, ra: ResolverTyAttr, item: ResolverTyItem) -> SynRes<TokenStream> {
        let g = Self {
            ty,
            name,
            ra,
            item,
        };

        let ty = &g.ty;
        let resolver = g.resolver_fn()?;
        let m = g.ty.to_string().to_snake_case().ts2_or_err()?;

        let r = quote! {
            mod #m {
                use super::*;

                #[derive(Default)]
                pub struct #ty;
                #[Object]
                impl #ty {
                    #resolver
                }
            }
            pub use #m::#ty;
        };

        #[cfg(feature = "debug_macro")]
        debug_macro(&g.item.gql_name, &r);

        Ok(r.into())
    }
}

impl AttrDebug for ResolverTy {
    fn attr_debug(&self) -> String {
        self.ra.inner.attr_debug()
    }
    fn span(&self) -> Span {
        self.ra.inner.span
    }
}

impl ResolverFn for ResolverTy {
    fn name(&self) -> SynRes<Ts2> {
        Ok(self.name.clone())
    }
    fn gql_name(&self) -> SynRes<String> {
        Ok(self.item.gql_name.clone())
    }
    fn inputs(&self) -> SynRes<Ts2> {
        Ok(self.item.inputs.clone())
    }
    fn output(&self) -> SynRes<Ts2> {
        Ok(self.item.output.clone())
    }
    fn body(&self) -> SynRes<Ts2> {
        Ok(self.item.body.clone())
    }

    fn db(&self) -> bool {
        self.ra.db
    }
    fn ctx(&self) -> bool {
        self.ra.ctx
    }
    fn check(&self) -> Vec<CheckAttr> {
        self.ra.check.clone()
    }
}
