#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

mod err;
mod libs;

pub mod export {
    pub use crate::err::MyErr as AuthUtilsErr;
    pub mod rand_utils {
        pub use crate::libs::*;
    }
}

pub mod reexport {
    pub use argon2;
    pub use base64;
    pub use hmac;
    pub use rand;
    pub use rand_core;
    pub use serde_qs;
    pub use sha2;
    pub use subtle;
    pub use zxcvbn;
}

pub mod prelude {
    pub use crate::export::*;
    pub use crate::reexport::*;

    pub(crate) use crate::export::AuthUtilsErr as MyErr;
    pub(crate) use crate::export::rand_utils::*;
    pub(crate) use _core::prelude::*;
}
