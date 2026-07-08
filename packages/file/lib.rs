#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

mod context;
#[cfg(feature = "ffmpeg")]
mod ffmpeg;
mod models;
mod resolvers;
mod schema;
#[cfg(feature = "test_util")]
mod test_util;
mod utils;

pub use utils::consts;

pub mod export {
    pub use crate::context::*;
    #[cfg(feature = "ffmpeg")]
    pub use crate::ffmpeg::*;
    pub use crate::models::*;
    pub use crate::resolvers::*;
    pub use crate::schema::*;
    #[cfg(feature = "test_util")]
    pub use crate::test_util::*;
    pub use crate::utils::*;
}

pub mod reexport {
    pub use aws_sdk_s3;
    #[cfg(feature = "test_util")]
    pub use aws_smithy_runtime;
    #[cfg(feature = "test_util")]
    pub use aws_smithy_types;
    #[cfg(feature = "test_util")]
    pub use http;
}

pub mod prelude {
    pub use crate::export::*;
    pub use crate::reexport::*;

    pub(crate) use crate::consts::*;
    pub(crate) use crate::export::FileErr as MyErr;
    pub(crate) use _core::prelude::*;
}
