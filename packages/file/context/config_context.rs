use crate::prelude::*;

static DEFAULT: LazyLock<FileConfig> = LazyLock::new(FileConfig::default);

/// Provides access to the configured FileConfig, falling back to defaults when none is set.
pub trait FileConfigContext<'a>
where
    Self: CoreContext<'a>,
{
    /// Returns the FileConfig registered on the context, or the default config if none was set.
    fn file_config(&self) -> &'a FileConfig {
        if let Some(cfg) = self.data_opt_impl::<FileConfig>() {
            cfg
        } else {
            &DEFAULT
        }
    }
}

impl<'a> FileConfigContext<'a> for Context<'a> {
}
