use super::prelude::*;

static DEFAULT: LazyLock<CoreConfig> = LazyLock::new(CoreConfig::default);

pub trait CoreConfigContext<'a>
where
    Self: ImplContext<'a>,
{
    /// Returns the CoreConfig from context, or a default CoreConfig if none was set.
    fn core_config(&self) -> &'a CoreConfig {
        if let Some(cfg) = self.data_opt_impl::<CoreConfig>() {
            cfg
        } else {
            &DEFAULT
        }
    }
}

impl<'a> CoreConfigContext<'a> for Context<'a> {
}
