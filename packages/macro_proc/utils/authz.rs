use crate::prelude::*;

pub fn gen_authz_row(filter: &Ts2, enable: bool) -> Ts2 {
    if !cfg!(feature = "authz") || !enable {
        return quote!(Option::<#filter>::None);
    }
    quote!(ctx.authz_row_graceful::<#filter>().await?)
}

pub fn gen_authz_row_def(filter: &Ts2, enable: bool) -> (Ts2, Ts2) {
    if !cfg!(feature = "authz") || !enable {
        return (quote!(Option::<#filter>::None), quote!());
    }
    let var = unique_ident();
    let authz_row = gen_authz_row(filter, enable);
    (var.clone(), quote!(let #var = #authz_row;))
}

pub fn gen_authz_err(enable: bool) -> Ts2 {
    if cfg!(feature = "authz") && enable {
        quote!(ctx.authz_err())
    } else {
        quote!(&CoreDbErr::Db404.into())
    }
}
