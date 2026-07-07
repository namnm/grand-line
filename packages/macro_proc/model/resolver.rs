use crate::prelude::*;

/// One #[resolver(...)] field of a model, ready to generate its resolver fn
/// by delegating to the user function named in a.f.
pub struct GenResolver {
    pub a: ResolverAttr,
    pub field_attrs: Vec<Attribute>,
}

impl VirtualResolverFn for GenResolver {
    fn sql_dep(&self) -> SynRes<Vec<String>> {
        Ok(self.a.sql_dep.clone())
    }
}
impl AttrDebug for GenResolver {
    fn attr_debug(&self) -> String {
        self.a.inner.attr_debug()
    }
    fn span(&self) -> Span {
        self.a.inner.span
    }
}

impl ResolverFn for GenResolver {
    fn tx(&self) -> bool {
        self.a.ra.tx
    }
    fn ctx(&self) -> bool {
        self.a.ra.ctx
    }
    fn name(&self) -> SynRes<Ts2> {
        self.a.inner.field_name()?.ts2_or_err()
    }
    fn gql_name(&self) -> SynRes<String> {
        let gql_name_override = attr_graphql_info(&self.field_attrs).0;
        if let Some(n) = gql_name_override {
            return Ok(n);
        }
        Ok(self.name()?.to_string().to_lower_camel_case())
    }
    fn docs(&self) -> Vec<String> {
        attr_docs(&self.field_attrs)
    }
    fn extra_graphql(&self) -> Ts2 {
        attr_graphql_info(&self.field_attrs).1
    }
    fn inputs(&self) -> SynRes<Ts2> {
        Ok(quote!())
    }
    fn output(&self) -> SynRes<Ts2> {
        self.a.inner.field_ty()?.ts2_or_err()
    }
    fn body(&self) -> SynRes<Ts2> {
        let f = &self.a.f;

        let r = if self.ctx() {
            quote!(#f(self, ctx).await?)
        } else {
            quote!(#f(self).await?)
        };
        Ok(r)
    }
}
