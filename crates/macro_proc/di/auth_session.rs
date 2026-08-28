use crate::prelude::*;

/// Proc-macro entry for #[auth_session], implements AuthSessionModel on the
/// entity so it can back the framework's default AuthSessionImpl.
pub fn gen_auth_session(attr: &TokenStream, item: TokenStream) -> TokenStream {
    try_gen_auth_session(attr, item).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_auth_session(attr: &TokenStream, item: TokenStream) -> SynRes<TokenStream> {
    let name = "auth_session";
    di_no_attr(name, attr)?;
    let m = DiModel::parse(name, item)?;
    m.require(&["id", "user_id", "secret_hashed", "created_at"])?;

    let impls = quote! {
        impl AuthSessionModel for Entity {
            fn auth_impl_session(m: Self::M) -> AuthImplSession {
                AuthImplSession {
                    id: m.id,
                    user_id: m.user_id,
                    secret_hashed: m.secret_hashed,
                    created_at: m.created_at,
                }
            }
        }
    };
    Ok(m.g(&impls))
}
