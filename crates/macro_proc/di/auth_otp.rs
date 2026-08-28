use crate::prelude::*;

/// Proc-macro entry for #[auth_otp], implements AuthOtpModel on the entity so
/// it can back the framework's default AuthOtpImpl.
pub fn gen_auth_otp(attr: &TokenStream, item: TokenStream) -> TokenStream {
    try_gen_auth_otp(attr, item).unwrap_or_else(|e| e.to_compile_error().into())
}

fn try_gen_auth_otp(attr: &TokenStream, item: TokenStream) -> SynRes<TokenStream> {
    let name = "auth_otp";
    di_no_attr(name, attr)?;
    let m = DiModel::parse(name, item)?;
    m.require(&[
        "id",
        "ty",
        "email",
        "secret_hashed",
        "otp_salt",
        "otp_hashed",
        "total_attempt",
        "data",
        "created_at",
    ])?;

    let impls = quote! {
        impl AuthOtpModel for Entity {
            fn col_ty() -> Self::C {
                Column::Ty
            }
            fn col_email() -> Self::C {
                Column::Email
            }
            fn col_total_attempt() -> Self::C {
                Column::TotalAttempt
            }
            fn auth_impl_otp(m: Self::M) -> AuthImplOtp {
                AuthImplOtp {
                    id: m.id,
                    email: m.email,
                    secret_hashed: m.secret_hashed,
                    otp_salt: m.otp_salt,
                    otp_hashed: m.otp_hashed,
                    total_attempt: m.total_attempt,
                    created_at: m.created_at,
                    data: m.data,
                }
            }
        }
    };
    Ok(m.g(&impls))
}
