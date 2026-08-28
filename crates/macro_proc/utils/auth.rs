use crate::prelude::*;

// The ctx aware db helpers live in the auth package, so without that feature
// the crud macros fall back to the plain ones. Which resolvers actually carry
// an actor is decided at runtime by ctx, not statically by the attribute.

/// Expression running an active model wrapper, ctx aware when available so the
/// audit actor is resolved from ctx.
pub fn gen_am_exec() -> Ts2 {
    if cfg!(feature = "auth") {
        quote!(exec(ctx))
    } else {
        quote!(exec_without_ctx(db))
    }
}

/// Same as gen_am_exec, for turning a wrapper into its plain active model.
pub fn gen_am_into() -> Ts2 {
    if cfg!(feature = "auth") {
        quote!(into_am(ctx).await?)
    } else {
        quote!(into_am_without_ctx())
    }
}

/// Expression for the actor to record on a history entry, None without auth.
pub fn gen_auth_by_id() -> Ts2 {
    if cfg!(feature = "auth") {
        quote!(ctx.auth().await.ok())
    } else {
        quote!(None)
    }
}
