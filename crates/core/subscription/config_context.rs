use super::prelude::*;

static DEFAULT: LazyLock<SubscriptionConfig> = LazyLock::new(SubscriptionConfig::default);

/// Provides access to the configured SubscriptionConfig, falling back to defaults when none is set.
pub trait SubscriptionConfigContext<'a>
where
    Self: BaseImplContext<'a>,
{
    /// Returns the SubscriptionConfig registered on the context, or the default one if none was set.
    fn subscription_config(&self) -> &'a SubscriptionConfig {
        if let Some(cfg) = self.data_opt_impl::<SubscriptionConfig>() {
            cfg
        } else {
            &DEFAULT
        }
    }
}

impl<'a> SubscriptionConfigContext<'a> for Context<'a> {
}

impl<'a> SubscriptionConfigContext<'a> for ExtensionContext<'a> {
}
