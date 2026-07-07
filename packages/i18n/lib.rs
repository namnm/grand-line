#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

mod libs;
mod utils;

pub mod export {
    pub use crate::libs::*;
    pub use crate::utils::*;
}

pub mod reexport {
    pub use fixed_decimal;
    pub use icu_calendar;
    pub use icu_datetime;
    pub use icu_decimal;
    pub use icu_locale_core;
    pub use icu_plurals;
    pub use icu_provider_blob;
}

pub mod prelude {
    pub use crate::export::*;
    pub use crate::reexport::*;

    pub(crate) use crate::export::I18mErr as MyErr;
    pub(crate) use _core::prelude::*;
}
