use crate::prelude::*;

/// Macros that change a row and therefore accept the publish flag.
pub static TY_PUBLISH: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let mut set = HashSet::new();
    set.insert(MacroTy::Create.to_string());
    set.insert(MacroTy::Update.to_string());
    set.insert(MacroTy::Delete.to_string());
    set
});

/// Statement queueing a row change for the subscription broker, empty when the
/// subscription feature is off. Queued only, the extension publishes it once the
/// request transaction has committed.
pub fn gen_subscription_queue(model: &Ts2, operation: &Ts2, id: &Ts2, enable: bool) -> Ts2 {
    if !cfg!(feature = "subscription") || !enable {
        return quote!();
    }
    quote!(ctx.subscription_queue::<#model>(SubscriptionOperation::#operation, #id).await?;)
}
