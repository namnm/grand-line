use crate::prelude::*;

/// Proc-macro entry for #[authz_org_id], marks the entity as org scoped so the
/// ctx.authz_org_* helpers can scope its rows to the current authz org.
pub fn gen_authz_org_id(attr: &TokenStream, item: TokenStream) -> TokenStream {
    try_gen_authz_org_id(attr, item).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_authz_org_id(attr: &TokenStream, item: TokenStream) -> SynRes<TokenStream> {
    let name = "authz_org_id";
    di_no_attr(name, attr)?;
    let m = DiModel::parse(name, item)?;
    m.require(&["org_id"])?;

    let impls = gen_impl_org_id();
    Ok(m.g(&impls))
}

/// The AuthzImplOrgId impl, shared with the role and user_in_role macros since
/// both of those models are org scoped too.
pub fn gen_impl_org_id() -> Ts2 {
    quote! {
        impl AuthzImplOrgId for Entity {
            fn col_org_id() -> Self::C {
                Column::OrgId
            }
        }
    }
}
