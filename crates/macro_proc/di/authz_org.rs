use crate::prelude::*;

/// Proc-macro entry for #[authz_org], marks the entity as the org lookup target
/// so it can back the framework's default AuthzOrgImpl.
pub fn gen_authz_org(attr: &TokenStream, item: TokenStream) -> TokenStream {
    try_gen_authz_org(attr, item).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_authz_org(attr: &TokenStream, item: TokenStream) -> SynRes<TokenStream> {
    let name = "authz_org";
    di_no_attr(name, attr)?;
    let m = DiModel::parse(name, item)?;

    let impls = quote! {
        impl AuthzOrg for Entity {
        }
    };
    Ok(m.g(&impls))
}
