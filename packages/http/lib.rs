#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

mod context;
mod utils;

pub use utils::consts;

pub mod export {
    pub use crate::context::*;
    pub use crate::utils::*;
    #[cfg(feature = "axum")]
    pub use _http_axum::export::*;
}

pub mod reexport {
    #[cfg(feature = "axum")]
    pub use _http_axum::reexport::*;
    pub use cookie;
}

pub mod prelude {
    pub use crate::export::*;
    pub use crate::reexport::*;
    #[cfg(feature = "axum")]
    pub use _http_axum::prelude::*;

    pub(crate) use crate::consts::*;
    pub(crate) use crate::export::HttpErr as MyErr;
    pub(crate) use _core::prelude::*;
}
