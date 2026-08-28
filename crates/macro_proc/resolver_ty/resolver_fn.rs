use crate::prelude::*;

/// Describes the pieces of one generated resolver method, implementors supply
/// the raw fn signature/body parts, resolver_fn assembles them into the final
/// async fn token stream with guard/db/ctx wiring applied.
pub trait ResolverFn
where
    Self: AttrDebug,
{
    fn name(&self) -> SynRes<Ts2>;
    fn gql_name(&self) -> SynRes<String>;
    fn inputs(&self) -> SynRes<Ts2>;
    fn output(&self) -> SynRes<Ts2>;
    fn body(&self) -> SynRes<Ts2>;

    /// Whether the generated resolver receives the db parameter, true by default.
    fn db(&self) -> bool {
        true
    }
    /// Whether the generated resolver receives the ctx parameter, true by default.
    fn ctx(&self) -> bool {
        true
    }
    /// Ctx guards to call in order before the body runs, empty means no guard.
    fn check(&self) -> Vec<CheckAttr> {
        vec![]
    }

    /// Doc-comment strings from the original field definition.
    /// Each entry corresponds to one /// line (with leading space preserved).
    fn docs(&self) -> Vec<String> {
        vec![]
    }

    /// Extra #[graphql(...)] args (everything except name) from the
    /// original field definition. Already formatted with trailing commas,
    /// ready to be spliced into the generated graphql attribute.
    fn extra_graphql(&self) -> Ts2 {
        quote!()
    }

    /// Builds the complete async fn token stream for this resolver: wraps the
    /// body with the declared ctx guards and the db connection as configured, injects
    /// ctx into the input list, wraps the output in Res<..>, and attaches the
    /// #[graphql(..)] attribute and doc comments.
    fn resolver_fn(&self) -> SynRes<Ts2> {
        let mut body = self.body()?;
        let ctx = self.ctx();

        let checks = self.check();
        if !checks.is_empty() {
            if !ctx {
                return Err(self.syn_err("check requires ctx"));
            }
            let calls = checks.iter().map(CheckAttr::call);
            body = quote! {
                #(#calls)*
                #body
            };
        }

        if self.db() {
            if !ctx {
                return Err(self.syn_err("db requires ctx"));
            }
            body = quote! {
                let db = &ctx.db().await?;
                #body
            };
        }

        let mut inputs = self.inputs()?;
        if ctx {
            inputs = quote!(ctx: &Context<'_>, #inputs);
        }

        let mut output = self.output()?;
        body = quote! {
            let r: #output = {
                #body
            };
            Ok(r)
        };
        output = quote!(Res<#output>);

        let name = self.name()?;
        let gql_name = self.gql_name()?;
        let extra = self.extra_graphql();
        let graphql = if extra.is_empty() {
            quote!(#[graphql(name = #gql_name)])
        } else {
            quote!(#[graphql(name = #gql_name, #extra)])
        };
        let docs = self.docs();

        let r = quote! {
            #graphql
            #(#[doc = #docs])*
            async fn #name(&self, #inputs) -> #output {
                #body
            }
        };
        Ok(r)
    }
}
