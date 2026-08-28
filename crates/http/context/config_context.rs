use crate::prelude::*;

static DEFAULT: LazyLock<HttpConfig> = LazyLock::new(HttpConfig::default);

/// Access to the HttpConfig attached to the schema context.
pub trait HttpConfigContext<'a>
where
    Self: ImplContext<'a>,
{
    /// Return the configured HttpConfig, or the default if none was set on the schema context.
    fn http_config(&self) -> &'a HttpConfig {
        if let Some(cfg) = self.data_opt_impl::<HttpConfig>() {
            cfg
        } else {
            &DEFAULT
        }
    }
}

impl<'a> HttpConfigContext<'a> for Context<'a> {
}
