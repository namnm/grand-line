use crate::prelude::*;

/// Proc-macro entry for #[authz_user_in_role], implements AuthzUserInRoleModel
/// on the entity so DefaultRoleImpl can resolve user to role assignments.
pub fn gen_authz_user_in_role(attr: &TokenStream, item: TokenStream) -> TokenStream {
    try_gen_authz_user_in_role(attr, item).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_authz_user_in_role(attr: &TokenStream, item: TokenStream) -> SynRes<TokenStream> {
    let name = "authz_user_in_role";
    di_no_attr(name, attr)?;
    let m = DiModel::parse(name, item)?;
    m.require(&["role_id", "user_id", "org_id"])?;

    let org_id = gen_impl_org_id();
    let impls = quote! {
        #org_id
        impl AuthzUserInRoleModel for Entity {
            fn col_role_id() -> Self::C {
                Column::RoleId
            }
            fn col_user_id() -> Self::C {
                Column::UserId
            }
        }
    };
    Ok(m.g(&impls))
}
