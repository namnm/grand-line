use crate::prelude::*;

/// Expression for the caller's authz row filter of the given type, evaluated
/// inline, or Option::<Filter>::None when the authz feature or authz_row is off.
pub fn gen_authz_row(filter: &Ts2, enable: bool) -> Ts2 {
    if !cfg!(feature = "authz") || !enable {
        return quote!(Option::<#filter>::None);
    }
    quote!(ctx.authz_row_graceful::<#filter>().await?)
}

/// Same as gen_authz_row, but returns (variable, let-binding statement) so the
/// row filter is evaluated once and can be reused (e.g. cloned) by the caller.
pub fn gen_authz_row_def(filter: &Ts2, enable: bool) -> (Ts2, Ts2) {
    if !cfg!(feature = "authz") || !enable {
        return (quote!(Option::<#filter>::None), quote!());
    }
    let var = unique_ident();
    let authz_row = gen_authz_row(filter, enable);
    (var.clone(), quote!(let #var = #authz_row;))
}

/// Error expression to raise when the authz row filter excludes a row: the
/// caller's configured authz_err when authz_row is enabled, otherwise a
/// plain Db404 so a filter miss looks like a missing row.
pub fn gen_authz_err(enable: bool) -> Ts2 {
    if cfg!(feature = "authz") && enable {
        quote!(ctx.authz_err())
    } else {
        quote!(&CoreDbErr::Db404.into())
    }
}
