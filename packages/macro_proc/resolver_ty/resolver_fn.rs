use crate::prelude::*;

pub trait ResolverFn
where
    Self: AttrDebug,
{
    fn name(&self) -> SynRes<Ts2>;
    fn gql_name(&self) -> SynRes<String>;
    fn inputs(&self) -> SynRes<Ts2>;
    fn output(&self) -> SynRes<Ts2>;
    fn body(&self) -> SynRes<Ts2>;

    fn root_operation_ty(&self) -> SynRes<Option<String>> {
        Ok(None)
    }

    fn tx(&self) -> bool {
        true
    }
    fn ctx(&self) -> bool {
        true
    }
    fn auth(&self) -> Option<AuthAttr> {
        None
    }
    fn authz(&self) -> Option<AuthzAttr> {
        None
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

    fn resolver_fn(&self) -> SynRes<Ts2> {
        let mut body = self.body()?;
        let ctx = self.ctx();

        if let Some(auth) = self.auth() {
            if !ctx {
                return Err(self.syn_err("auth requires ctx"));
            }
            let check = if auth.unauthenticated {
                "unauthenticated"
            } else {
                "authenticated"
            };
            let pascal = check.to_pascal_case().ts2_or_err()?;
            let ensure = quote!(AuthEnsure::#pascal);
            body = quote! {
                ctx.auth_ensure_in_macro(#ensure).await?;
                #body
            };
        }

        if let Some(authz) = self.authz() {
            if !ctx {
                return Err(self.syn_err("authz requires ctx"));
            }
            self.root_operation_ty()?
                .ok_or_else(|| self.syn_err("authz only available in root resolvers"))?;
            let realm = authz.realm;
            let org = !authz.skip_org;
            let user = !authz.skip_user;
            let ensure = quote! {
                AuthzEnsure {
                    realm: #realm.to_owned(),
                    org: #org,
                    user: #user,
                }
            };
            body = quote! {
                ctx.authz_ensure_in_macro(#ensure).await?;
                #body
            };
        }

        let tx = self.tx();
        if tx {
            if !ctx {
                return Err(self.syn_err("tx requires ctx"));
            }
            body = quote! {
                let tx = &*ctx.tx().await?;
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
